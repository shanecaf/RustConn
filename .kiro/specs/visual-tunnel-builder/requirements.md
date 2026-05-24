# Requirements Document

## Introduction

Візуальний конструктор SSH-тунелів (Visual Tunnel Builder) — інтерактивний UI-компонент для RustConn, який замінює поточний плоский діалог створення/редагування тунелів на wizard-подібний інтерфейс із візуальним представленням шляху тунелю (localhost → bastion → target). Конструктор робить налаштування складних ланцюжків SSH port forwarding (-L, -R, -D) та ProxyJump інтуїтивно зрозумілим, з індикацією стану тунелю в реальному часі. Дотримується GNOME HIG та Libadwaita-first підходу.

## Glossary

- **Tunnel_Builder**: Візуальний wizard-діалог для створення та редагування SSH-тунелів, реалізований як `adw::Dialog`
- **Tunnel_Path_Diagram**: Візуальна схема шляху тунелю, що показує ланцюжок вузлів (localhost → bastion → target) з індикацією стану
- **Port_Forward_Rule**: Правило перенаправлення порту — Local (-L), Remote (-R) або Dynamic (-D) — представлене структурою `PortForward`
- **Bastion_Host**: Проміжний SSH-сервер (jump host), через який встановлюється з'єднання до кінцевого сервера у приватній підмережі
- **Tunnel_Chain**: Повний ланцюжок з'єднання: локальний хост → один або кілька bastion-хостів → кінцевий сервер
- **Status_Indicator**: Візуальний індикатор стану тунелю (Running, Starting, Failed, Stopped) на діаграмі шляху
- **Tunnel_Manager_Window**: Існуюче вікно керування тунелями (`TunnelManagerWindow`), яке відкриває Tunnel_Builder замість старого діалогу
- **SSH_Connection**: Існуюче з'єднання SSH у RustConn, яке надає хост, порт, ім'я користувача, ключ та jump host конфігурацію

## Requirements

### Requirement 1: Wizard-діалог створення тунелю

**User Story:** Як користувач, я хочу створювати SSH-тунелі через покроковий wizard-діалог, щоб складна конфігурація була розбита на зрозумілі етапи.

#### Acceptance Criteria

1. WHEN користувач натискає кнопку "Add Tunnel" у Tunnel_Manager_Window, THE Tunnel_Builder SHALL відкрити wizard-діалог із 3 послідовними кроками: (1) вибір SSH-з'єднання та введення назви тунелю, (2) налаштування port forwarding правил та опцій (auto-start, auto-reconnect), (3) перегляд конфігурації та підтвердження
2. THE Tunnel_Builder SHALL відображати навігаційну панель (stepper) із назвами всіх кроків та візуальним виділенням поточного кроку
3. WHEN користувач перебуває на будь-якому кроці wizard (крім першого), THE Tunnel_Builder SHALL надавати кнопку "Назад" для повернення до попереднього кроку зі збереженням раніше введених даних
4. WHEN користувач натискає кнопку переходу до наступного кроку, THE Tunnel_Builder SHALL валідувати поточний крок перед переходом: крок 1 вимагає непорожню назву тунелю (від 1 до 128 символів) та обране SSH-з'єднання; крок 2 дозволяє 0 або більше Port_Forward_Rule (максимум 32 правила)
5. IF валідація поточного кроку не пройдена, THEN THE Tunnel_Builder SHALL залишити користувача на поточному кроці та візуально позначити поля з помилками
6. WHEN користувач натискає кнопку "Створити" на останньому кроці wizard, THE Tunnel_Builder SHALL зберегти новий `StandaloneTunnel` у налаштуваннях та оновити список у Tunnel_Manager_Window
7. IF користувач закриває wizard-діалог без збереження, THEN THE Tunnel_Builder SHALL відхилити всі незбережені зміни без модифікації стану застосунку

### Requirement 2: Вибір SSH-з'єднання з підтримкою bastion

**User Story:** Як користувач, я хочу обрати SSH-з'єднання та опціонально вказати bastion-хост, щоб тунель міг проходити через проміжні сервери.

#### Acceptance Criteria

1. THE Tunnel_Builder SHALL відображати список SSH-з'єднань (protocol = SSH) із SharedAppState із фільтрацією за підрядком у назві (`name`) або хості (`host`), нечутливою до регістру, що оновлюється при кожному введеному символі
2. WHEN користувач обирає SSH_Connection, що має налаштований `jump_host_id` або `proxy_jump`, THE Tunnel_Builder SHALL автоматично відобразити Bastion_Host у Tunnel_Path_Diagram
3. IF обране SSH_Connection має `jump_host_id`, що посилається на неіснуюче з'єднання у SharedAppState, THEN THE Tunnel_Builder SHALL відобразити попередження про відсутній jump host та дозволити користувачу обрати інший bastion вручну
4. WHEN користувач обирає SSH_Connection без bastion, THE Tunnel_Builder SHALL надавати опцію додати Bastion_Host вручну через окремий випадаючий список з'єднань (з тим самим фільтром), що використовується як jump host
5. THE Tunnel_Builder SHALL підтримувати ланцюжки з максимум одним Bastion_Host (localhost → bastion → target), відображаючи лише перший hop навіть якщо jump host має власний `jump_host_id`
6. IF список SSH-з'єднань порожній, THEN THE Tunnel_Builder SHALL відобразити повідомлення з пропозицією створити нове SSH-з'єднання та кнопку для відкриття діалогу створення з'єднання

### Requirement 3: Візуальна діаграма шляху тунелю

**User Story:** Як користувач, я хочу бачити візуальну схему шляху тунелю, щоб розуміти, як дані проходять від локального порту до кінцевого сервера.

#### Acceptance Criteria

1. THE Tunnel_Path_Diagram SHALL відображати ланцюжок вузлів у горизонтальному вигляді: локальний хост (із зазначенням імені хосту та порту) → Bastion_Host (якщо налаштований, із зазначенням імені хосту) → кінцевий сервер (із зазначенням імені хосту та порту)
2. THE Tunnel_Path_Diagram SHALL з'єднувати вузли стрілками, що вказують напрямок потоку даних від джерела до призначення
3. WHEN тунель має тип Local (-L), THE Tunnel_Path_Diagram SHALL показувати стрілку від вузла локального хосту (з номером локального порту) через Bastion_Host (якщо є) до вузла віддаленого хосту (з номером віддаленого порту)
4. WHEN тунель має тип Remote (-R), THE Tunnel_Path_Diagram SHALL показувати стрілку від вузла віддаленого хосту (з номером віддаленого порту) через Bastion_Host (якщо є) до вузла локального хосту (з номером локального порту)
5. WHEN тунель має тип Dynamic (-D), THE Tunnel_Path_Diagram SHALL показувати вузол локального хосту з номером порту та позначкою "SOCKS proxy", стрілку через Bastion_Host (якщо є) до вузла SSH-сервера
6. WHEN користувач змінює параметри тунелю у wizard (тип, порти, з'єднання), THE Tunnel_Path_Diagram SHALL оновити відображення протягом 300 мілісекунд після зміни
7. IF жодне SSH_Connection не обрано або обов'язкові поля порту порожні, THEN THE Tunnel_Path_Diagram SHALL відображати вузли-заповнювачі з позначками, що вказують на відсутні дані, без стрілок напрямку

### Requirement 4: Налаштування правил port forwarding

**User Story:** Як користувач, я хочу додавати, редагувати та видаляти правила port forwarding у візуальному інтерфейсі, щоб керувати перенаправленням портів без ручного введення SSH-аргументів.

#### Acceptance Criteria

1. THE Tunnel_Builder SHALL надавати кнопку додавання нового Port_Forward_Rule, яка створює новий рядок із випадаючим списком вибору типу (Local, Remote, Dynamic) та полями введення параметрів
2. WHEN користувач обирає тип Local або Remote, THE Tunnel_Builder SHALL відображати поля: локальний порт (adw::SpinRow, діапазон 1–65535), віддалений хост (adw::EntryRow, максимум 253 символи), віддалений порт (adw::SpinRow, діапазон 1–65535)
3. WHEN користувач обирає тип Dynamic, THE Tunnel_Builder SHALL відображати лише поле локального порту (adw::SpinRow, діапазон 1–65535) та приховати поля віддаленого хосту і порту
4. THE Tunnel_Builder SHALL дозволяти додавати до 20 Port_Forward_Rule до одного тунелю
5. THE Tunnel_Builder SHALL надавати кнопку видалення на кожному Port_Forward_Rule, яка видаляє це правило зі списку без підтвердження
6. IF користувач вводить номер порту поза діапазоном 1–65535, THEN THE Tunnel_Builder SHALL заборонити збереження тунелю та відобразити повідомлення про помилку біля відповідного поля порту
7. IF користувач вводить порт менше 1024, THEN THE Tunnel_Builder SHALL відобразити попередження біля поля порту, що привілейовані порти можуть потребувати підвищених прав, без блокування збереження
8. WHEN користувач змінює параметри існуючого Port_Forward_Rule (тип, порти, хост), THE Tunnel_Builder SHALL оновити заголовок рядка правила для відображення поточної конфігурації у форматі "L 8080 → host:80", "R 3306 → db:3306" або "D 1080 (SOCKS)"
9. IF користувач обирає тип Local або Remote і залишає поле віддаленого хосту порожнім, THEN THE Tunnel_Builder SHALL заборонити збереження тунелю та відобразити повідомлення про помилку біля поля віддаленого хосту

### Requirement 5: Індикація стану тунелю

**User Story:** Як користувач, я хочу бачити поточний стан тунелю на візуальній діаграмі, щоб швидко розуміти, чи працює тунель.

#### Acceptance Criteria

1. WHILE тунель має статус Running, THE Status_Indicator SHALL застосовувати CSS-клас `success` до вузлів Tunnel_Path_Diagram та відображати анімовану лінію з'єднання між вузлами
2. WHILE тунель має статус Starting, THE Status_Indicator SHALL застосовувати CSS-клас `warning` до вузлів Tunnel_Path_Diagram та відображати пульсуючу анімацію на індикаторі
3. WHILE тунель має статус Failed, THE Status_Indicator SHALL застосовувати CSS-клас `error` до вузлів Tunnel_Path_Diagram та відображати у tooltip текст помилки зі значення `Failed(String)`, обрізаний до максимум 200 символів
4. WHILE тунель має статус Stopped, THE Status_Indicator SHALL застосовувати неактивний стиль (dim/insensitive) до вузлів Tunnel_Path_Diagram без анімації
5. WHEN статус тунелю змінюється, THE Status_Indicator SHALL оновити візуальне представлення протягом 500 мілісекунд та оголосити новий статус для assistive technologies через ATK accessible property
6. WHILE Tunnel_Builder відкритий у режимі створення нового тунелю (без існуючого StandaloneTunnel), THE Status_Indicator SHALL бути прихованим на Tunnel_Path_Diagram

### Requirement 6: Редагування існуючого тунелю

**User Story:** Як користувач, я хочу редагувати існуючий тунель через той самий візуальний конструктор, щоб мати єдиний інтерфейс для створення та модифікації.

#### Acceptance Criteria

1. WHEN користувач натискає "Edit" на існуючому тунелі у Tunnel_Manager_Window, THE Tunnel_Builder SHALL відкрити wizard-діалог із попередньо заповненими даними тунелю: обране SSH-з'єднання, усі Port_Forward_Rule та назва тунелю
2. WHEN Tunnel_Builder відкривається в режимі редагування, THE Tunnel_Path_Diagram SHALL відображати поточну конфігурацію тунелю із вузлами та з'єднаннями відповідно до збережених Port_Forward_Rule
3. WHEN користувач зберігає зміни в режимі редагування, THE Tunnel_Builder SHALL оновити існуючий `StandaloneTunnel` у налаштуваннях, зберігаючи його UUID, та оновити відповідний рядок у списку Tunnel_Manager_Window
4. IF тунель має статус Running на момент натискання кнопки "Save" у режимі редагування, THEN THE Tunnel_Builder SHALL відобразити попередження, що зміни набудуть чинності після перезапуску тунелю, та дозволити користувачу підтвердити або скасувати збереження
5. IF SSH-з'єднання, прив'язане до тунелю, було видалене з налаштувань, THEN THE Tunnel_Builder SHALL відобразити повідомлення про відсутнє з'єднання та вимагати вибору нового SSH-з'єднання перед збереженням

### Requirement 7: Генерація SSH-команди (preview)

**User Story:** Як користувач, я хочу бачити згенеровану SSH-команду перед збереженням тунелю, щоб перевірити правильність конфігурації.

#### Acceptance Criteria

1. THE Tunnel_Builder SHALL відображати на фінальному кроці wizard згенеровану SSH-команду у моноширинному текстовому блоці, що включає: прапорець `-N`, аргументи port forwarding (`-L`, `-R`, `-D`), опцію порту (`-p`), та призначення (`user@host`)
2. WHEN користувач змінює будь-який параметр тунелю (з'єднання, port forwarding правила, bastion host), THE Tunnel_Builder SHALL оновити preview SSH-команди протягом 1 секунди після зміни
3. WHEN користувач натискає кнопку копіювання, THE Tunnel_Builder SHALL скопіювати повний текст SSH-команди до системного буфера обміну та відобразити тимчасове підтвердження успішного копіювання
4. WHEN SSH_Connection має Bastion_Host, THE Tunnel_Builder SHALL включити аргумент `-J` із значенням у форматі `user@host:port` bastion-сервера у preview команди
5. IF жодного Port_Forward_Rule не додано до тунелю, THEN THE Tunnel_Builder SHALL відображати preview команди без аргументів `-L`/`-R`/`-D` та показувати повідомлення, що правила port forwarding не налаштовані

### Requirement 8: Доступність та i18n

**User Story:** Як користувач із обмеженими можливостями, я хочу мати повний доступ до функціональності конструктора тунелів через допоміжні технології.

#### Acceptance Criteria

1. THE Tunnel_Builder SHALL забезпечити кожну icon-only кнопку tooltip із текстовим описом дії та ATK accessible label з ідентичним текстом для screen readers
2. THE Tunnel_Builder SHALL підтримувати навігацію клавіатурою: Tab та Shift+Tab переміщують фокус між інтерактивними елементами в межах поточного кроку wizard, Enter активує кнопку переходу до наступного кроку, а при зміні кроку фокус переміщується на перший інтерактивний елемент нового кроку
3. THE Tunnel_Path_Diagram SHALL мати ATK accessible description, що містить перелік вузлів ланцюжка із зазначенням імені хосту, порту та поточного статусу кожного вузла, та оновлювати цей опис при зміні конфігурації або статусу тунелю
4. THE Tunnel_Builder SHALL використовувати `i18n()` або `i18n_f()` для всіх рядків, видимих користувачу, включаючи tooltip, accessible label та повідомлення про помилки валідації
5. THE Tunnel_Builder SHALL використовувати семантичні кольори Libadwaita (CSS-класи `.success`, `.warning`, `.error`) замість жорстко закодованих кольорів для Status_Indicator
6. WHEN виникає помилка валідації або змінюється статус тунелю, THE Tunnel_Builder SHALL оголосити зміну через ATK live region (gtk::Accessible notify), щоб screen readers повідомили користувача без переміщення фокусу

### Requirement 9: Інтеграція з існуючою архітектурою

**User Story:** Як розробник, я хочу, щоб Visual Tunnel Builder інтегрувався з існуючою архітектурою RustConn, щоб зберегти консистентність коду.

#### Acceptance Criteria

1. THE Tunnel_Builder SHALL використовувати існуючу модель `StandaloneTunnel` з `rustconn-core` без модифікації її полів (`id`, `name`, `connection_id`, `forwards`, `auto_start`, `auto_reconnect`, `enabled`)
2. THE Tunnel_Builder SHALL використовувати існуючий `SharedTunnelManager` (методи `start`, `stop`, `status`) для запуску та зупинки тунелів, не створюючи власних механізмів управління процесами
3. THE Tunnel_Builder SHALL розміщуватися у `rustconn/src/dialogs/` як окремий модуль та бути зареєстрованим у `dialogs/mod.rs` через `pub mod` та відповідний `pub use`
4. THE Tunnel_Builder SHALL використовувати `SharedAppState` для читання списку з'єднань (`connections`) та збереження тунелів у `settings.standalone_tunnels`
5. THE Tunnel_Builder SHALL не містити логіки побудови SSH-аргументів, валідації портів чи управління процесами — лише код побудови GTK4-віджетів, обробки подій UI та виклики до `rustconn-core`
6. IF операція `SharedTunnelManager` повертає помилку (`TunnelManagerError`), THEN THE Tunnel_Builder SHALL відобразити користувачу повідомлення про помилку із зазначенням причини збою
7. THE Tunnel_Builder SHALL не імпортувати та не використовувати жодних типів з `gtk4`, `adw` або `vte4` у файлах крейту `rustconn-core`
