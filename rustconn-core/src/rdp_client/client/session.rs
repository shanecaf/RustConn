#[cfg(feature = "gfx-h264")]
use super::super::gfx_handler::GfxFrameUpdate;
use super::super::{RdpClientCommand, RdpClientError, RdpClientEvent, RdpRect};
use super::commands::process_command;
use super::connection::UpgradedFramed;
use ironrdp::connector::ConnectionResult;
use ironrdp::connector::connection_activation::{
    ConnectionActivationFactory, ConnectionActivationState,
};
use ironrdp::graphics::image_processing::PixelFormat as IronPixelFormat;
use ironrdp::pdu::WriteBuf;
use ironrdp::session::ActiveStageBuilder;
use ironrdp::session::image::DecodedImage;
use ironrdp::session::{ActiveStage, ActiveStageOutput, fast_path};
use ironrdp_tokio::{
    Framed, FramedRead, FramedWrite, single_sequence_step_read, split_tokio_framed,
};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// Runs the active RDP session, processing framebuffer updates and input
///
/// When the `gfx-h264` feature is enabled, `gfx_update_rx` carries decoded
/// EGFX frame updates from the `GraphicsPipelineHandler`. The session loop
/// drains it after each `ActiveStage::process()` call to convert RGBA→BGRA
/// and emit `FrameUpdate` events to the GUI.
pub async fn run_active_session(
    framed: UpgradedFramed,
    connection_result: ConnectionResult,
    event_tx: std::sync::mpsc::Sender<RdpClientEvent>,
    command_rx: std::sync::mpsc::Receiver<RdpClientCommand>,
    shutdown_signal: Arc<AtomicBool>,
    #[cfg(feature = "gfx-h264")] gfx_update_rx: std::sync::mpsc::Receiver<GfxFrameUpdate>,
) -> Result<(), RdpClientError> {
    let (mut reader, mut writer) = split_tokio_framed(framed);

    // Create decoded image buffer
    let mut image = DecodedImage::new(
        IronPixelFormat::BgrA32,
        connection_result.desktop_size.width,
        connection_result.desktop_size.height,
    );

    // Performance monitoring: FrameStatistics tracks decode times and drop rates
    let mut frame_stats = super::super::graphics::FrameStatistics::new();
    // Adaptive polling: tracks frames received since last idle timeout.
    // Resets to 0 on timeout (idle), incremented on each PDU received.
    let mut frames_since_last_idle: u32 = 0;
    // Set active graphics mode based on feature availability
    #[cfg(feature = "gfx-h264")]
    {
        frame_stats.active_graphics_mode = super::super::graphics::GraphicsMode::GfxH264;
    }
    #[cfg(not(feature = "gfx-h264"))]
    {
        frame_stats.active_graphics_mode = super::super::graphics::GraphicsMode::RemoteFx;
    }

    // Build ActiveStage from ConnectionResult fields (ironrdp 0.17 builder pattern)
    let activation_factory = connection_result.activation_factory;
    let mut active_stage = ActiveStageBuilder {
        static_channels: connection_result.static_channels,
        user_channel_id: connection_result.user_channel_id,
        io_channel_id: connection_result.io_channel_id,
        message_channel_id: connection_result.message_channel_id,
        share_id: connection_result.share_id,
        compression_type: connection_result.compression_type,
        enable_server_pointer: connection_result.enable_server_pointer,
        pointer_software_rendering: connection_result.pointer_software_rendering,
    }
    .build();

    loop {
        // Check shutdown signal
        if shutdown_signal.load(Ordering::SeqCst) {
            if let Ok(frames) = active_stage.graceful_shutdown() {
                for frame in frames {
                    if let ActiveStageOutput::ResponseFrame(data) = frame {
                        let _ = writer.write_all(&data).await;
                    }
                }
            }
            break;
        }

        // Process commands from GUI (non-blocking)
        while let Ok(cmd) = command_rx.try_recv() {
            if process_command(cmd, &mut active_stage, &mut image, &mut writer, &event_tx).await? {
                return Ok(());
            }
        }

        // Read and process RDP frames with adaptive timeout.
        // When the server is actively sending frames (e.g. animation, user
        // interaction), we poll at ~60 FPS (16ms) to keep latency low. When
        // idle (timeout expired with no data), we back off to 50ms to reduce
        // CPU usage from busy-waiting on a static desktop.
        // ponytail: true async select! on command_rx needs tokio::sync::mpsc;
        // left as-is (std::sync::mpsc) to avoid a cross-cutting refactor.
        let idle_timeout = if frames_since_last_idle > 0 {
            std::time::Duration::from_millis(16) // active: low latency
        } else {
            std::time::Duration::from_millis(50) // idle: save CPU
        };

        let read_result = tokio::time::timeout(idle_timeout, reader.read_pdu()).await;

        match read_result {
            Ok(Ok((action, payload))) => {
                frames_since_last_idle = frames_since_last_idle.saturating_add(1);
                match active_stage.process(&mut image, action, &payload) {
                    Ok(outputs) => {
                        for output in outputs {
                            if handle_active_stage_output(
                                output,
                                &mut writer,
                                &mut reader,
                                &event_tx,
                                &mut image,
                                &mut active_stage,
                                &activation_factory,
                                &frame_stats,
                            )
                            .await?
                            {
                                return Ok(());
                            }
                        }

                        // Drain decoded GFX frame updates produced during process().
                        // The GraphicsPipelineHandler fires on_bitmap_updated() inside
                        // ActiveStage::process() and sends updates via the mpsc channel.
                        #[cfg(feature = "gfx-h264")]
                        drain_gfx_updates(&gfx_update_rx, &mut image, &event_tx, &mut frame_stats);
                    }
                    Err(e) => {
                        return Err(RdpClientError::ProtocolError(format!("Session error: {e}")));
                    }
                }
            }
            Ok(Err(e)) => {
                return Err(RdpClientError::ConnectionFailed(format!("Read error: {e}")));
            }
            Err(_) => {
                // Timeout - no data available, enter idle mode
                frames_since_last_idle = 0;
            }
        }
    }

    Ok(())
}

#[expect(
    clippy::too_many_arguments,
    reason = "internal dispatch function — parameters are all distinct; grouping into a struct adds indirection without clarity"
)]
async fn handle_active_stage_output<S>(
    output: ActiveStageOutput,
    writer: &mut impl FramedWrite,
    reader: &mut Framed<S>,
    event_tx: &std::sync::mpsc::Sender<RdpClientEvent>,
    image: &mut DecodedImage,
    active_stage: &mut ActiveStage,
    activation_factory: &ConnectionActivationFactory,
    frame_stats: &super::super::graphics::FrameStatistics,
) -> Result<bool, RdpClientError>
where
    S: FramedRead + Unpin + Send,
{
    match output {
        ActiveStageOutput::ResponseFrame(data) => {
            if let Err(e) = writer.write_all(&data).await {
                return Err(RdpClientError::ConnectionFailed(format!(
                    "Write error: {e}"
                )));
            }
        }
        ActiveStageOutput::GraphicsUpdate(region) => {
            let rect = RdpRect::new(
                region.left,
                region.top,
                region.right.saturating_sub(region.left),
                region.bottom.saturating_sub(region.top),
            );
            let data = extract_region_data(image, rect);
            let _ = event_tx.send(RdpClientEvent::FrameUpdate { rect, data });
        }
        ActiveStageOutput::PointerDefault => {
            let _ = event_tx.send(RdpClientEvent::CursorDefault);
        }
        ActiveStageOutput::PointerHidden => {
            let _ = event_tx.send(RdpClientEvent::CursorHidden);
        }
        ActiveStageOutput::PointerPosition { x, y } => {
            let _ = event_tx.send(RdpClientEvent::CursorPosition { x, y });
        }
        ActiveStageOutput::PointerBitmap(pointer) => {
            let expected_size = usize::from(pointer.width) * usize::from(pointer.height) * 4;

            let src = if pointer.bitmap_data.len() > expected_size {
                &pointer.bitmap_data[..expected_size]
            } else {
                &pointer.bitmap_data
            };

            let data = src.to_vec();

            // Pass RGBA data as-is — handle_cursor_update crops transparent
            // padding and does premultiplied alpha + R↔B for HiDPI cursors
            // (pointer bitmaps from IronRDP are RGBA, unlike framebuffer which is BGRA)
            let _ = event_tx.send(RdpClientEvent::CursorUpdate {
                width: pointer.width,
                height: pointer.height,
                hotspot_x: pointer.hotspot_x,
                hotspot_y: pointer.hotspot_y,
                data,
            });
        }
        ActiveStageOutput::Terminate(reason) => {
            tracing::info!("RDP session terminated: {reason:?}");
            return Ok(true);
        }
        ActiveStageOutput::DeactivateAll => {
            handle_reactivation(
                activation_factory,
                reader,
                writer,
                image,
                active_stage,
                event_tx,
            )
            .await?;
        }
        ActiveStageOutput::MultitransportRequest(pdu) => {
            // IronRDP 0.15: server requests sideband UDP transport.
            // We do not implement UDP multitransport — log and continue.
            tracing::debug!(
                request_id = pdu.request_id,
                "Server requested multitransport (UDP) — not supported, ignoring"
            );
        }
        ActiveStageOutput::AutoDetect(request) => {
            // IronRDP 0.16: server sends network characteristics result.
            // Extract RTT measurement and forward to GUI.
            if let ironrdp::pdu::rdp::autodetect::AutoDetectRequest::NetworkCharacteristicsResult {
                average_rtt_ms,
                ..
            } = &request
            {
                let _ = event_tx.send(RdpClientEvent::Rtt {
                    rtt_ms: *average_rtt_ms,
                    active_graphics_mode: frame_stats.active_graphics_mode,
                });
            }
            tracing::debug!(
                ?request,
                "Received Auto-Detect network characteristics from server"
            );
        }
    }
    Ok(false)
}

async fn handle_reactivation<S>(
    activation_factory: &ConnectionActivationFactory,
    reader: &mut Framed<S>,
    writer: &mut impl FramedWrite,
    image: &mut DecodedImage,
    active_stage: &mut ActiveStage,
    event_tx: &std::sync::mpsc::Sender<RdpClientEvent>,
) -> Result<(), RdpClientError>
where
    S: FramedRead + Unpin + Send,
{
    // Execute the Deactivation-Reactivation Sequence:
    // https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-rdpbcgr/dfc234ce-481a-4674-9a5d-2a7bafb14432
    tracing::debug!(
        "Received Server Deactivate All PDU, executing Deactivation-Reactivation Sequence"
    );

    let mut connection_activation = activation_factory.create();
    let io_channel_id = activation_factory.io_channel_id();
    let user_channel_id = activation_factory.user_channel_id();

    let mut buf = WriteBuf::new();
    loop {
        let written =
            match single_sequence_step_read(reader, &mut connection_activation, &mut buf).await {
                Ok(w) => w,
                Err(e) => {
                    tracing::warn!("Reactivation sequence error: {}", e);
                    break;
                }
            };

        if written.size().is_some()
            && let Err(e) = writer.write_all(buf.filled()).await
        {
            tracing::warn!("Failed to send reactivation response: {}", e);
            break;
        }

        if let ConnectionActivationState::Finalized {
            desktop_size,
            share_id,
            enable_server_pointer,
            pointer_software_rendering,
        } = connection_activation.connection_activation_state()
        {
            tracing::debug!(
                ?desktop_size,
                "Deactivation-Reactivation Sequence completed"
            );

            // Update image size with the new desktop size
            *image = DecodedImage::new(
                IronPixelFormat::BgrA32,
                desktop_size.width,
                desktop_size.height,
            );

            // Update the active stage with new channel IDs
            // and pointer settings
            active_stage.set_fastpath_processor(
                fast_path::ProcessorBuilder {
                    io_channel_id,
                    user_channel_id,
                    share_id,
                    enable_server_pointer,
                    pointer_software_rendering,
                    // Bulk compression is disabled at connection time
                    // (`compression_type: None`), so the server never sends
                    // compressed FastPath data and no decompressor is needed.
                    // See the note in `connection.rs` and issue #200 for why
                    // reactivation + bulk compression cannot be reconciled with
                    // the current ironrdp-session API.
                    bulk_decompressor: None,
                }
                .build(),
            );
            // Update share_id if the server assigned a new one
            active_stage.set_share_id(share_id);
            active_stage.set_enable_server_pointer(enable_server_pointer);

            // Notify GUI about resolution change
            let _ = event_tx.send(RdpClientEvent::ResolutionChanged {
                width: desktop_size.width,
                height: desktop_size.height,
            });

            break;
        }
    }
    Ok(())
}

/// Extracts pixel data for a specific region from the decoded image.
///
/// IronRDP 0.16 outputs pixels in BgrA32 which matches Cairo's ARGB32 format
/// on little-endian (both are B-G-R-A byte order in memory). No channel swap needed.
///
/// Optimized for 4K rendering: uses row-based `memcpy` which is cache-friendly
/// and auto-vectorizable by LLVM.
fn extract_region_data(image: &DecodedImage, rect: RdpRect) -> Vec<u8> {
    let img_width = image.width();
    let img_height = image.height();
    let data = image.data();

    let region_x = rect.x.min(img_width);
    let region_y = rect.y.min(img_height);
    let region_w = rect.width.min(img_width.saturating_sub(region_x));
    let region_h = rect.height.min(img_height.saturating_sub(region_y));

    if region_w == 0 || region_h == 0 {
        return Vec::new();
    }

    let bpp = 4;

    // Fast path: if the region covers the entire image, avoid row-by-row copy
    if region_x == 0 && region_y == 0 && region_w == img_width && region_h == img_height {
        return data.to_vec();
    }

    let src_stride = img_width as usize * bpp;
    let dst_stride = region_w as usize * bpp;
    let result_size = dst_stride * region_h as usize;
    let mut result = vec![0u8; result_size];

    // Copy rows in bulk (cache-friendly, compiles to memcpy)
    for row in 0..region_h as usize {
        let src_offset = (region_y as usize + row) * src_stride + region_x as usize * bpp;
        let dst_offset = row * dst_stride;

        if src_offset + dst_stride <= data.len() {
            result[dst_offset..dst_offset + dst_stride]
                .copy_from_slice(&data[src_offset..src_offset + dst_stride]);
        }
    }

    result
}

// ============================================================================
// GFX Pipeline Integration (gfx-h264 feature)
// ============================================================================

/// Drains pending GFX frame updates and sends them as `FrameUpdate` events.
///
/// Called after `ActiveStage::process()` returns — the `GraphicsPipelineHandler`
/// fires during `process()` and enqueues decoded RGBA frames via the mpsc channel.
/// We convert RGBA→BGRA and send directly to the GUI without blitting into
/// `DecodedImage` (which has no mutable data accessor in the public API).
///
/// A sentinel update with empty `data` signals a resolution change from
/// `on_reset_graphics` — the framebuffer is resized and the GUI is notified.
///
/// Bounds checking ensures updates that exceed the framebuffer dimensions are
/// clipped to avoid panics.
#[cfg(feature = "gfx-h264")]
fn drain_gfx_updates(
    gfx_update_rx: &std::sync::mpsc::Receiver<GfxFrameUpdate>,
    image: &mut DecodedImage,
    event_tx: &std::sync::mpsc::Sender<RdpClientEvent>,
    frame_stats: &mut super::super::graphics::FrameStatistics,
) {
    while let Ok(update) = gfx_update_rx.try_recv() {
        // Sentinel: empty data with non-zero dimensions = resolution reset
        if update.data.is_empty() {
            if update.width > 0 && update.height > 0 {
                *image = DecodedImage::new(IronPixelFormat::BgrA32, update.width, update.height);
                let _ = event_tx.send(RdpClientEvent::ResolutionChanged {
                    width: update.width,
                    height: update.height,
                });
                tracing::debug!(
                    width = update.width,
                    height = update.height,
                    "GFX reset: framebuffer resized"
                );
            }
            continue;
        }

        // Skip zero-dimension updates
        if update.width == 0 || update.height == 0 {
            continue;
        }

        let img_width = image.width();
        let img_height = image.height();

        // Clip update region to framebuffer bounds
        let clipped_w = if update.x >= img_width {
            continue;
        } else {
            update.width.min(img_width.saturating_sub(update.x))
        };
        let clipped_h = if update.y >= img_height {
            continue;
        } else {
            update.height.min(img_height.saturating_sub(update.y))
        };

        // Measure RGBA→BGRA conversion time
        let blit_start = std::time::Instant::now();
        let bgra_data = convert_gfx_rgba_to_bgra(&update, clipped_w, clipped_h);
        let blit_elapsed_us = blit_start.elapsed().as_micros() as u64;

        // Update H.264 decode/blit time EMA
        frame_stats.update_h264_decode_time(blit_elapsed_us);

        let rect = RdpRect::new(update.x, update.y, clipped_w, clipped_h);
        let _ = event_tx.send(RdpClientEvent::FrameUpdate {
            rect,
            data: bgra_data,
        });
    }
}

/// Converts RGBA pixel data from a GFX frame update to BGRA format.
///
/// Performs row-by-row conversion with R↔B channel swap, respecting the
/// clipped dimensions (which may be smaller than the original update when
/// the update extends beyond the framebuffer boundary).
///
/// # Performance
///
/// The inner loop processes 4 bytes per pixel sequentially. LLVM auto-vectorizes
/// this pattern for SSE2/AVX on x86_64.
#[cfg(feature = "gfx-h264")]
fn convert_gfx_rgba_to_bgra(update: &GfxFrameUpdate, clipped_w: u16, clipped_h: u16) -> Vec<u8> {
    let src_stride = usize::from(update.width) * 4;
    let dst_pixel_count = usize::from(clipped_w) * usize::from(clipped_h);
    let mut result = Vec::with_capacity(dst_pixel_count * 4);

    for row in 0..usize::from(clipped_h) {
        let src_row_start = row * src_stride;

        for px in 0..usize::from(clipped_w) {
            let si = src_row_start + px * 4;

            // Bounds check: skip if source data is insufficient
            if si + 3 >= update.data.len() {
                // Pad remaining pixels with opaque black
                result.extend_from_slice(&[0, 0, 0, 255]);
                continue;
            }

            // RGBA → BGRA: swap R and B channels
            result.push(update.data[si + 2]); // B
            result.push(update.data[si + 1]); // G
            result.push(update.data[si]); // R
            result.push(update.data[si + 3]); // A
        }
    }

    result
}
