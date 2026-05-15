#!/usr/bin/env python3
"""Translate untranslated entries in PO files for pl, cs, sk."""

# Polish/Czech/Slovak use: opening \u201e and closing \u201d
OQ = "\u201e"  # opening low double quote
CQ = "\u201d"  # closing right double quote

translations = {
    "Automatic Reconnection": {
        "pl": "Automatyczne ponowne \u0142\u0105czenie",
        "cs": "Automatick\u00e9 op\u011btovn\u00e9 p\u0159ipojen\u00ed",
        "sk": "Automatick\u00e9 op\u00e4tovn\u00e9 pripojenie",
    },
    "Retry connection with exponential backoff on failure": {
        "pl": "Pon\u00f3w po\u0142\u0105czenie z wyk\u0142adniczym op\u00f3\u017anieniem po awarii",
        "cs": "Opakovat p\u0159ipojen\u00ed s exponenci\u00e1ln\u00edm zpo\u017ed\u011bn\u00edm p\u0159i selh\u00e1n\u00ed",
        "sk": "Opakova\u0165 pripojenie s exponenci\u00e1lnym oneskoren\u00edm pri zlyhan\u00ed",
    },
    "Enable auto-reconnect": {
        "pl": "W\u0142\u0105cz automatyczne ponowne \u0142\u0105czenie",
        "cs": "Povolit automatick\u00e9 op\u011btovn\u00e9 p\u0159ipojen\u00ed",
        "sk": "Povoli\u0165 automatick\u00e9 op\u00e4tovn\u00e9 pripojenie",
    },
    "Automatically retry when connection drops": {
        "pl": "Automatycznie pon\u00f3w pr\u00f3b\u0119 po utracie po\u0142\u0105czenia",
        "cs": "Automaticky opakovat p\u0159i p\u0159eru\u0161en\u00ed spojen\u00ed",
        "sk": "Automaticky opakova\u0165 pri preru\u0161en\u00ed spojenia",
    },
    "Maximum attempts": {
        "pl": "Maksymalna liczba pr\u00f3b",
        "cs": "Maxim\u00e1ln\u00ed po\u010det pokus\u016f",
        "sk": "Maxim\u00e1lny po\u010det pokusov",
    },
    "Number of reconnection attempts before giving up": {
        "pl": "Liczba pr\u00f3b ponownego po\u0142\u0105czenia przed rezygnacj\u0105",
        "cs": "Po\u010det pokus\u016f o op\u011btovn\u00e9 p\u0159ipojen\u00ed p\u0159ed vzd\u00e1n\u00edm",
        "sk": "Po\u010det pokusov o op\u00e4tovn\u00e9 pripojenie pred vzdan\u00edm",
    },
    "Initial delay (ms)": {
        "pl": "Pocz\u0105tkowe op\u00f3\u017anienie (ms)",
        "cs": "Po\u010d\u00e1te\u010dn\u00ed zpo\u017ed\u011bn\u00ed (ms)",
        "sk": "Po\u010diato\u010dn\u00e9 oneskorenie (ms)",
    },
    "Delay before first reconnection attempt": {
        "pl": "Op\u00f3\u017anienie przed pierwsz\u0105 pr\u00f3b\u0105 ponownego po\u0142\u0105czenia",
        "cs": "Zpo\u017ed\u011bn\u00ed p\u0159ed prvn\u00edm pokusem o op\u011btovn\u00e9 p\u0159ipojen\u00ed",
        "sk": "Oneskorenie pred prv\u00fdm pokusom o op\u00e4tovn\u00e9 pripojenie",
    },
    "Maximum delay (ms)": {
        "pl": "Maksymalne op\u00f3\u017anienie (ms)",
        "cs": "Maxim\u00e1ln\u00ed zpo\u017ed\u011bn\u00ed (ms)",
        "sk": "Maxim\u00e1lne oneskorenie (ms)",
    },
    "Upper limit for backoff delay between attempts": {
        "pl": "G\u00f3rny limit op\u00f3\u017anienia mi\u0119dzy pr\u00f3bami",
        "cs": "Horn\u00ed limit zpo\u017ed\u011bn\u00ed mezi pokusy",
        "sk": "Horn\u00fd limit oneskorenia medzi pokusmi",
    },
    "ProxyCommand": {
        "pl": "ProxyCommand",
        "cs": "ProxyCommand",
        "sk": "ProxyCommand",
    },
    "Custom proxy command (e.g., for .onion hosts)": {
        "pl": "W\u0142asne polecenie proxy (np. dla host\u00f3w .onion)",
        "cs": "Vlastn\u00ed p\u0159\u00edkaz proxy (nap\u0159. pro hostitele .onion)",
        "sk": "Vlastn\u00fd pr\u00edkaz proxy (napr. pre hostite\u013eov .onion)",
    },
    "Multiple Files (batch)": {
        "pl": "Wiele plik\u00f3w (wsadowo)",
        "cs": "V\u00edce soubor\u016f (d\u00e1vkov\u011b)",
        "sk": "Viacero s\u00faborov (d\u00e1vkovo)",
    },
    "Import connections from multiple files at once": {
        "pl": "Importuj po\u0142\u0105czenia z wielu plik\u00f3w jednocze\u015bnie",
        "cs": "Importovat p\u0159ipojen\u00ed z v\u00edce soubor\u016f najednou",
        "sk": "Importova\u0165 pripojenia z viacer\u00fdch s\u00faborov naraz",
    },
    "Multiple Files": {
        "pl": "Wiele plik\u00f3w",
        "cs": "V\u00edce soubor\u016f",
        "sk": "Viacero s\u00faborov",
    },
    "Select Files to Import": {
        "pl": "Wybierz pliki do importu",
        "cs": "Vyberte soubory k importu",
        "sk": "Vyberte s\u00fabory na import",
    },
    "All supported formats": {
        "pl": "Wszystkie obs\u0142ugiwane formaty",
        "cs": "V\u0161echny podporovan\u00e9 form\u00e1ty",
        "sk": "V\u0161etky podporovan\u00e9 form\u00e1ty",
    },
    "Importing from {} files...": {
        "pl": "Importowanie z {} plik\u00f3w\u2026",
        "cs": "Importov\u00e1n\u00ed z {} soubor\u016f\u2026",
        "sk": "Importovanie z {} s\u00faborov\u2026",
    },
    "Importing {}...": {
        "pl": "Importowanie {}\u2026",
        "cs": "Importov\u00e1n\u00ed {}\u2026",
        "sk": "Importovanie {}\u2026",
    },
    "Custom command for the local shell tab": {
        "pl": "W\u0142asne polecenie dla karty lokalnej pow\u0142oki",
        "cs": "Vlastn\u00ed p\u0159\u00edkaz pro kartu m\u00edstn\u00edho shellu",
        "sk": "Vlastn\u00fd pr\u00edkaz pre kartu miestneho shellu",
    },
    "Default system shell": {
        "pl": "Domy\u015blna pow\u0142oka systemowa",
        "cs": "V\u00fdchoz\u00ed syst\u00e9mov\u00fd shell",
        "sk": "Predvolen\u00fd syst\u00e9mov\u00fd shell",
    },
    "e.g. fish, bash --norc, neofetch &amp;&amp; bash": {
        "pl": "np. fish, bash --norc, neofetch &amp;&amp; bash",
        "cs": "nap\u0159. fish, bash --norc, neofetch &amp;&amp; bash",
        "sk": "napr. fish, bash --norc, neofetch &amp;&amp; bash",
    },
    "Authentication failed: invalid username or password.": {
        "pl": "Uwierzytelnianie nie powiod\u0142o si\u0119: nieprawid\u0142owa nazwa u\u017cytkownika lub has\u0142o.",
        "cs": "Ov\u011b\u0159en\u00ed selhalo: neplatn\u00e9 u\u017eivatelsk\u00e9 jm\u00e9no nebo heslo.",
        "sk": "Overenie zlyhalo: neplatn\u00e9 pou\u017e\u00edvate\u013esk\u00e9 meno alebo heslo.",
    },
    "Authentication failed: account is disabled.": {
        "pl": "Uwierzytelnianie nie powiod\u0142o si\u0119: konto jest wy\u0142\u0105czone.",
        "cs": "Ov\u011b\u0159en\u00ed selhalo: \u00fa\u010det je zak\u00e1z\u00e1n.",
        "sk": "Overenie zlyhalo: \u00fa\u010det je zak\u00e1zan\u00fd.",
    },
    "Authentication failed: account is locked out.": {
        "pl": "Uwierzytelnianie nie powiod\u0142o si\u0119: konto jest zablokowane.",
        "cs": "Ov\u011b\u0159en\u00ed selhalo: \u00fa\u010det je uzam\u010den.",
        "sk": "Overenie zlyhalo: \u00fa\u010det je uzamknut\u00fd.",
    },
    "Authentication failed: password has expired.": {
        "pl": "Uwierzytelnianie nie powiod\u0142o si\u0119: has\u0142o wygas\u0142o.",
        "cs": "Ov\u011b\u0159en\u00ed selhalo: platnost hesla vypr\u0161ela.",
        "sk": "Overenie zlyhalo: platnos\u0165 hesla vypr\u0161ala.",
    },
    "Authentication failed: account has expired.": {
        "pl": "Uwierzytelnianie nie powiod\u0142o si\u0119: konto wygas\u0142o.",
        "cs": "Ov\u011b\u0159en\u00ed selhalo: platnost \u00fa\u010dtu vypr\u0161ela.",
        "sk": "Overenie zlyhalo: platnos\u0165 \u00fa\u010dtu vypr\u0161ala.",
    },
    "Authentication failed: user is not allowed to log on to this computer.": {
        "pl": "Uwierzytelnianie nie powiod\u0142o si\u0119: u\u017cytkownik nie ma uprawnie\u0144 do logowania na tym komputerze.",
        "cs": "Ov\u011b\u0159en\u00ed selhalo: u\u017eivatel nem\u00e1 opr\u00e1vn\u011bn\u00ed p\u0159ihl\u00e1sit se k tomuto po\u010d\u00edta\u010di.",
        "sk": "Overenie zlyhalo: pou\u017e\u00edvate\u013e nem\u00e1 opr\u00e1vnenie prihl\u00e1si\u0165 sa na tomto po\u010d\u00edta\u010di.",
    },
    "NLA authentication failed. Check username and password.": {
        "pl": "Uwierzytelnianie NLA nie powiod\u0142o si\u0119. Sprawd\u017a nazw\u0119 u\u017cytkownika i has\u0142o.",
        "cs": "Ov\u011b\u0159en\u00ed NLA selhalo. Zkontrolujte u\u017eivatelsk\u00e9 jm\u00e9no a heslo.",
        "sk": "Overenie NLA zlyhalo. Skontrolujte pou\u017e\u00edvate\u013esk\u00e9 meno a heslo.",
    },
    "TLS connection failed. The server may not support this security level.": {
        "pl": "Po\u0142\u0105czenie TLS nie powiod\u0142o si\u0119. Serwer mo\u017ce nie obs\u0142ugiwa\u0107 tego poziomu zabezpiecze\u0144.",
        "cs": "P\u0159ipojen\u00ed TLS selhalo. Server nemus\u00ed podporovat tuto \u00farove\u0148 zabezpe\u010den\u00ed.",
        "sk": "Pripojenie TLS zlyhalo. Server nemus\u00ed podporova\u0165 t\u00fato \u00farove\u0148 zabezpe\u010denia.",
    },
    "Connection refused. Check host and port.": {
        "pl": "Po\u0142\u0105czenie odrzucone. Sprawd\u017a host i port.",
        "cs": "P\u0159ipojen\u00ed odm\u00edtnuto. Zkontrolujte hostitele a port.",
        "sk": "Pripojenie odmietnut\u00e9. Skontrolujte hostite\u013ea a port.",
    },
    "Connection timed out. Check that the host is reachable.": {
        "pl": "Przekroczono limit czasu po\u0142\u0105czenia. Sprawd\u017a, czy host jest osi\u0105galny.",
        "cs": "\u010casov\u00fd limit p\u0159ipojen\u00ed vypr\u0161el. Zkontrolujte, zda je hostitel dostupn\u00fd.",
        "sk": "\u010casov\u00fd limit pripojenia vypr\u0161al. Skontrolujte, \u010di je hostite\u013e dostupn\u00fd.",
    },
    "Cloud Sync across devices": {
        "pl": "Synchronizacja w chmurze mi\u0119dzy urz\u0105dzeniami",
        "cs": "Cloudov\u00e1 synchronizace mezi za\u0159\u00edzen\u00edmi",
        "sk": "Cloudov\u00e1 synchroniz\u00e1cia medzi zariadeniami",
    },
    "Drag & drop files to sessions": {
        "pl": "Przeci\u0105gnij i upu\u015b\u0107 pliki do sesji",
        "cs": "P\u0159et\u00e1hn\u011bte soubory do relac\u00ed",
        "sk": "Presu\u0148te s\u00fabory do rel\u00e1ci\u00ed",
    },
    "Tab Overview grid (Ctrl+Shift+O)": {
        "pl": "Siatka przegl\u0105du kart (Ctrl+Shift+O)",
        "cs": "M\u0159\u00ed\u017eka p\u0159ehledu karet (Ctrl+Shift+O)",
        "sk": "Mrie\u017eka preh\u013eadu kariet (Ctrl+Shift+O)",
    },
    "Imported {} connections and {} snippets to '{}' group": {
        "pl": "Zaimportowano {} po\u0142\u0105cze\u0144 i {} fragment\u00f3w do grupy \u201e{}\u201d",
        "cs": "Importov\u00e1no {} p\u0159ipojen\u00ed a {} \u00faryvk\u016f do skupiny \u201e{}\u201d",
        "sk": "Importovan\u00fdch {} pripojen\u00ed a {} \u00faryvkov do skupiny \u201e{}\u201d",
    },
    "Imported {} connections to '{}' group": {
        "pl": "Zaimportowano {} po\u0142\u0105cze\u0144 do grupy \u201e{}\u201d",
        "cs": "Importov\u00e1no {} p\u0159ipojen\u00ed do skupiny \u201e{}\u201d",
        "sk": "Importovan\u00fdch {} pripojen\u00ed do skupiny \u201e{}\u201d",
    },
}

# Multiline translations: key is a substring to match in the full msgid
# Value is dict of lang -> list of msgstr continuation lines
multiline_translations = [
    {
        "match": "Imported {} connection(s) and {} group(s) from {} files ({} errors)",
        "pl": [
            "Zaimportowano {} po\u0142\u0105cze\u0144 i {} grup z {} plik\u00f3w ({} b\u0142\u0119d\u00f3w).\\n",
            "Po\u0142\u0105czenia zostan\u0105 dodane do grupy \u201eMultiple Files Import\u201d.",
        ],
        "cs": [
            "Importov\u00e1no {} p\u0159ipojen\u00ed a {} skupin z {} soubor\u016f ({} chyb).\\n",
            "P\u0159ipojen\u00ed budou p\u0159id\u00e1na do skupiny \u201eMultiple Files Import\u201d.",
        ],
        "sk": [
            "Importovan\u00fdch {} pripojen\u00ed a {} skup\u00edn z {} s\u00faborov ({} ch\u00fdb).\\n",
            "Pripojenia bud\u00fa pridan\u00e9 do skupiny \u201eMultiple Files Import\u201d.",
        ],
    },
    {
        "match": "Imported {} connection(s) and {} group(s) from {} files.\\n",
        "pl": [
            "Zaimportowano {} po\u0142\u0105cze\u0144 i {} grup z {} plik\u00f3w.\\n",
            "Po\u0142\u0105czenia zostan\u0105 dodane do grupy \u201eMultiple Files Import\u201d.",
        ],
        "cs": [
            "Importov\u00e1no {} p\u0159ipojen\u00ed a {} skupin z {} soubor\u016f.\\n",
            "P\u0159ipojen\u00ed budou p\u0159id\u00e1na do skupiny \u201eMultiple Files Import\u201d.",
        ],
        "sk": [
            "Importovan\u00fdch {} pripojen\u00ed a {} skup\u00edn z {} s\u00faborov.\\n",
            "Pripojenia bud\u00fa pridan\u00e9 do skupiny \u201eMultiple Files Import\u201d.",
        ],
    },
    {
        "match": "Create a template to quickly set up new connections with predefined settings.",
        "pl": [
            "Utw\u00f3rz szablon, aby szybko konfigurowa\u0107 nowe po\u0142\u0105czenia ",
            "z predefiniowanymi ustawieniami.",
        ],
        "cs": [
            "Vytvo\u0159te \u0161ablonu pro rychl\u00e9 nastaven\u00ed nov\u00fdch p\u0159ipojen\u00ed ",
            "s p\u0159eddefinovan\u00fdmi nastaven\u00edmi.",
        ],
        "sk": [
            "Vytvorte \u0161abl\u00f3nu na r\u00fdchle nastavenie nov\u00fdch pripojen\u00ed ",
            "s preddefinovan\u00fdmi nastaveniami.",
        ],
    },
    {
        "match": "External file managers cannot access SSH agent",
        "pl": [
            "Zewn\u0119trzne mened\u017cery plik\u00f3w nie maj\u0105 dost\u0119pu do agenta SSH ",
            "we Flatpaku. W\u0142\u0105cz \u201eSFTP przez mc\u201d w Ustawieniach, aby ",
            "uzyska\u0107 niezawodny dost\u0119p.",
        ],
        "cs": [
            "Extern\u00ed spr\u00e1vci soubor\u016f nemaj\u00ed p\u0159\u00edstup k SSH agentovi ",
            "ve Flatpaku. Povolte \u201eSFTP p\u0159es mc\u201d v Nastaven\u00ed pro ",
            "spolehliv\u00fd p\u0159\u00edstup.",
        ],
        "sk": [
            "Extern\u00e9 spr\u00e1vcovia s\u00faborov nemaj\u00fa pr\u00edstup k SSH agentovi ",
            "vo Flatpaku. Povo\u013ete \u201eSFTP cez mc\u201d v Nastaveniach pre ",
            "spo\u013eahliv\u00fd pr\u00edstup.",
        ],
    },
]


def translate_file(filepath, lang):
    with open(filepath, "r", encoding="utf-8") as f:
        lines = f.readlines()

    i = 0
    changes = 0
    result = []

    while i < len(lines):
        line = lines[i]

        # Check for single-line msgid
        if line.startswith('msgid "') and not line.startswith('msgid ""'):
            msgid_content = line[7:].rstrip("\n")[:-1]  # strip 'msgid "' prefix and trailing '"'
            result.append(line)
            i += 1

            # Check if next line is empty msgstr
            if i < len(lines) and lines[i].strip() == 'msgstr ""':
                # Check if the line after msgstr is NOT a continuation
                if i + 1 >= len(lines) or not lines[i + 1].startswith('"'):
                    if msgid_content in translations and lang in translations[msgid_content]:
                        trans = translations[msgid_content][lang]
                        result.append(f'msgstr "{trans}"\n')
                        changes += 1
                        i += 1
                        continue
            result.append(lines[i])
            i += 1
            continue

        # Check for multi-line msgid (msgid ""\n"first line...)
        if line.strip() == 'msgid ""':
            result.append(line)
            i += 1
            # Collect all msgid continuation lines
            msgid_parts = []
            while i < len(lines) and lines[i].startswith('"'):
                content_line = lines[i].rstrip("\n")[1:-1]  # strip quotes
                msgid_parts.append(content_line)
                result.append(lines[i])
                i += 1

            # Now i points to msgstr line
            if i < len(lines) and lines[i].strip() == 'msgstr ""':
                # Check if msgstr is empty (no continuation lines after it)
                has_continuation = (i + 1 < len(lines) and lines[i + 1].startswith('"'))

                if not has_continuation and msgid_parts:
                    full_msgid = "".join(msgid_parts)

                    # Try multiline translations first
                    matched = False
                    for mt in multiline_translations:
                        if mt["match"] in full_msgid:
                            if lang in mt:
                                trans_lines = mt[lang]
                                result.append('msgstr ""\n')
                                for tl in trans_lines:
                                    result.append(f'"{tl}"\n')
                                changes += 1
                                matched = True
                                break

                    if not matched:
                        # Try single-line translations
                        if full_msgid in translations and lang in translations[full_msgid]:
                            trans = translations[full_msgid][lang]
                            result.append(f'msgstr "{trans}"\n')
                            changes += 1
                            matched = True

                    if matched:
                        i += 1  # skip the original msgstr ""
                        continue

            result.append(lines[i])
            i += 1
            continue

        result.append(line)
        i += 1

    with open(filepath, "w", encoding="utf-8") as f:
        f.writelines(result)

    print(f"{filepath}: {changes} translations added")


if __name__ == "__main__":
    translate_file("po/pl.po", "pl")
    translate_file("po/cs.po", "cs")
    translate_file("po/sk.po", "sk")
