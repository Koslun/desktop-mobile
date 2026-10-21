//! Keyboard input mapping — shared, platform-agnostic building blocks.

use std::collections::BTreeSet;

use super::executor::ModifierState;

/// The WebDriver null key; in Element Send Keys it releases every held modifier.
pub const NULL_KEY: &str = "\u{E000}";

/// Maps a WebDriver special-key value (U+E000–U+E05D) to `(key, code, keyCode)`.
///
/// `key` and `code` follow the W3C normalized-key and code tables; `keyCode` is the legacy value a
/// US keyboard reports. Every code point in the range maps, so none is ever typed as text.
pub fn map_special_key(key: &str) -> Option<(&'static str, &'static str, u32)> {
    let mut chars = key.chars();
    let (Some(ch), None) = (chars.next(), chars.next()) else {
        return None;
    };
    let mapped = match ch {
        '\u{E000}' => ("Unidentified", "", 0),
        '\u{E001}' => ("Cancel", "", 3),
        '\u{E002}' => ("Help", "Help", 47),
        '\u{E003}' => ("Backspace", "Backspace", 8),
        '\u{E004}' => ("Tab", "Tab", 9),
        '\u{E005}' => ("Clear", "", 12),
        // The spec's code table puts the main Enter key on U+E006 ("Return") and NumpadEnter on
        // U+E007, but clients send U+E007 for Enter, so it keeps the main key's code.
        '\u{E006}' => ("Enter", "NumpadEnter", 13),
        '\u{E007}' => ("Enter", "Enter", 13),
        '\u{E008}' => ("Shift", "ShiftLeft", 16),
        '\u{E009}' => ("Control", "ControlLeft", 17),
        '\u{E00A}' => ("Alt", "AltLeft", 18),
        '\u{E00B}' => ("Pause", "Pause", 19),
        '\u{E00C}' => ("Escape", "Escape", 27),
        '\u{E00D}' => (" ", "Space", 32),
        '\u{E00E}' => ("PageUp", "PageUp", 33),
        '\u{E00F}' => ("PageDown", "PageDown", 34),
        '\u{E010}' => ("End", "End", 35),
        '\u{E011}' => ("Home", "Home", 36),
        '\u{E012}' => ("ArrowLeft", "ArrowLeft", 37),
        '\u{E013}' => ("ArrowUp", "ArrowUp", 38),
        '\u{E014}' => ("ArrowRight", "ArrowRight", 39),
        '\u{E015}' => ("ArrowDown", "ArrowDown", 40),
        '\u{E016}' => ("Insert", "Insert", 45),
        '\u{E017}' => ("Delete", "Delete", 46),
        // The spec's code table has no row for U+E018; this is the US-layout `;` key.
        '\u{E018}' => (";", "Semicolon", 186),
        '\u{E019}' => ("=", "NumpadEqual", 187),
        '\u{E01A}' => ("0", "Numpad0", 96),
        '\u{E01B}' => ("1", "Numpad1", 97),
        '\u{E01C}' => ("2", "Numpad2", 98),
        '\u{E01D}' => ("3", "Numpad3", 99),
        '\u{E01E}' => ("4", "Numpad4", 100),
        '\u{E01F}' => ("5", "Numpad5", 101),
        '\u{E020}' => ("6", "Numpad6", 102),
        '\u{E021}' => ("7", "Numpad7", 103),
        '\u{E022}' => ("8", "Numpad8", 104),
        '\u{E023}' => ("9", "Numpad9", 105),
        '\u{E024}' => ("*", "NumpadMultiply", 106),
        '\u{E025}' => ("+", "NumpadAdd", 107),
        '\u{E026}' => (",", "NumpadComma", 108),
        '\u{E027}' => ("-", "NumpadSubtract", 109),
        '\u{E028}' => (".", "NumpadDecimal", 110),
        '\u{E029}' => ("/", "NumpadDivide", 111),
        '\u{E031}' => ("F1", "F1", 112),
        '\u{E032}' => ("F2", "F2", 113),
        '\u{E033}' => ("F3", "F3", 114),
        '\u{E034}' => ("F4", "F4", 115),
        '\u{E035}' => ("F5", "F5", 116),
        '\u{E036}' => ("F6", "F6", 117),
        '\u{E037}' => ("F7", "F7", 118),
        '\u{E038}' => ("F8", "F8", 119),
        '\u{E039}' => ("F9", "F9", 120),
        '\u{E03A}' => ("F10", "F10", 121),
        '\u{E03B}' => ("F11", "F11", 122),
        '\u{E03C}' => ("F12", "F12", 123),
        '\u{E03D}' => ("Meta", "MetaLeft", 91),
        '\u{E040}' => ("ZenkakuHankaku", "", 0),
        '\u{E050}' => ("Shift", "ShiftRight", 16),
        '\u{E051}' => ("Control", "ControlRight", 17),
        '\u{E052}' => ("Alt", "AltRight", 18),
        '\u{E053}' => ("Meta", "MetaRight", 92),
        '\u{E054}' => ("PageUp", "Numpad9", 33),
        '\u{E055}' => ("PageDown", "Numpad3", 34),
        '\u{E056}' => ("End", "Numpad1", 35),
        '\u{E057}' => ("Home", "Numpad7", 36),
        '\u{E058}' => ("ArrowLeft", "Numpad4", 37),
        '\u{E059}' => ("ArrowUp", "Numpad8", 38),
        '\u{E05A}' => ("ArrowRight", "Numpad6", 39),
        '\u{E05B}' => ("ArrowDown", "Numpad2", 40),
        '\u{E05C}' => ("Insert", "Numpad0", 45),
        '\u{E05D}' => ("Delete", "NumpadDecimal", 46),
        // Code points the spec leaves unassigned still dispatch as a key rather than as text.
        '\u{E000}'..='\u{E05D}' => ("Unidentified", "", 0),
        _ => return None, // printable character
    };
    Some(mapped)
}

/// Whether a normalized key value types a character: Space, `;`, `=` and the numpad digits and
/// operators.
pub fn is_printable_key(js_key: &str) -> bool {
    js_key.chars().count() == 1
}

/// Whether a WebDriver key value is a left- or right-hand Shift/Control/Alt/Meta.
pub fn is_modifier_key(key: &str) -> bool {
    matches!(
        map_special_key(key),
        Some(("Shift" | "Control" | "Alt" | "Meta", _, _))
    )
}

/// The `KeyboardEvent.code` for a printable character.
pub fn regular_key_code_for(key: &str) -> String {
    let ch = key.chars().next().unwrap_or(' ');
    let upper = ch.to_ascii_uppercase();
    if ch.is_ascii_alphabetic() {
        format!("Key{upper}")
    } else if ch.is_ascii_digit() {
        format!("Digit{ch}")
    } else {
        key.to_string()
    }
}

/// JS dispatching a synthetic special-key `KeyboardEvent` plus the value/selection/radio
/// side-effects an untrusted event can't drive itself.
pub fn build_special_key_script(
    js_key: &str,
    js_code: &str,
    key_code: u32,
    is_down: bool,
    modifiers: &ModifierState,
) -> String {
    let event_type = if is_down { "keydown" } else { "keyup" };
    let ctrl_key = modifiers.ctrl;
    let meta_key = modifiers.meta;
    let shift_key = modifiers.shift;
    let alt_key = modifiers.alt;

    if is_down && (js_key == "Backspace" || js_key == "Delete") {
        format!(
            r"(function() {{
                    var activeEl = document.activeElement || document.body;

                    var keydownEvent = new KeyboardEvent('keydown', {{
                        key: '{js_key}',
                        code: '{js_code}',
                        keyCode: {key_code},
                        which: {key_code},
                        ctrlKey: {ctrl_key},
                        metaKey: {meta_key},
                        shiftKey: {shift_key},
                        altKey: {alt_key},
                        bubbles: true,
                        cancelable: true
                    }});
                    activeEl.dispatchEvent(keydownEvent);

                    if (activeEl.tagName === 'INPUT' || activeEl.tagName === 'TEXTAREA') {{
                        var nativeInputValueSetter = Object.getOwnPropertyDescriptor(
                            activeEl.tagName === 'INPUT'
                                ? window.HTMLInputElement.prototype
                                : window.HTMLTextAreaElement.prototype,
                            'value'
                        ).set;

                        var currentValue = activeEl.value;
                        var selStart = activeEl.selectionStart;
                        var selEnd = activeEl.selectionEnd;
                        var newValue;
                        var inputType;

                        if (selStart !== selEnd) {{
                            newValue = currentValue.slice(0, selStart) + currentValue.slice(selEnd);
                            inputType = 'deleteContentBackward';
                            nativeInputValueSetter.call(activeEl, newValue);
                            activeEl.setSelectionRange(selStart, selStart);
                        }} else if ('{js_key}' === 'Backspace' && selStart > 0) {{
                            newValue = currentValue.slice(0, selStart - 1) + currentValue.slice(selStart);
                            inputType = 'deleteContentBackward';
                            nativeInputValueSetter.call(activeEl, newValue);
                            activeEl.setSelectionRange(selStart - 1, selStart - 1);
                        }} else if ('{js_key}' === 'Delete' && selStart < currentValue.length) {{
                            newValue = currentValue.slice(0, selStart) + currentValue.slice(selStart + 1);
                            inputType = 'deleteContentForward';
                            nativeInputValueSetter.call(activeEl, newValue);
                            activeEl.setSelectionRange(selStart, selStart);
                        }} else {{
                            return true;
                        }}

                        var inputEvent = new InputEvent('input', {{
                            bubbles: true,
                            cancelable: true,
                            inputType: inputType
                        }});
                        activeEl.dispatchEvent(inputEvent);
                    }}

                    return true;
                }})()"
        )
    } else if is_down
        && (js_key == "ArrowDown"
            || js_key == "ArrowUp"
            || js_key == "ArrowLeft"
            || js_key == "ArrowRight")
    {
        let go_forward = js_key == "ArrowDown" || js_key == "ArrowRight";
        format!(
            r#"(function() {{
                    var activeEl = document.activeElement || document.body;

                    var keydownEvent = new KeyboardEvent('keydown', {{
                        key: '{js_key}',
                        code: '{js_code}',
                        keyCode: {key_code},
                        which: {key_code},
                        ctrlKey: {ctrl_key},
                        metaKey: {meta_key},
                        shiftKey: {shift_key},
                        altKey: {alt_key},
                        bubbles: true,
                        cancelable: true
                    }});
                    activeEl.dispatchEvent(keydownEvent);

                    if (activeEl.tagName === 'INPUT' && activeEl.type === 'radio' && activeEl.name) {{
                        var name = activeEl.name;
                        var radios = Array.from(document.querySelectorAll("input[type='radio'][name='" + name + "']"));
                        var currentIndex = radios.indexOf(activeEl);

                        if (currentIndex !== -1 && radios.length > 1) {{
                            var nextIndex;
                            if ({go_forward}) {{
                                nextIndex = (currentIndex + 1) % radios.length;
                            }} else {{
                                nextIndex = (currentIndex - 1 + radios.length) % radios.length;
                            }}

                            var nextRadio = radios[nextIndex];
                            nextRadio.checked = true;
                            nextRadio.focus();

                            var changeEvent = new Event('change', {{ bubbles: true }});
                            nextRadio.dispatchEvent(changeEvent);
                        }}
                    }}

                    return true;
                }})()"#
        )
    } else {
        format!(
            r"(function() {{
                    var event = new KeyboardEvent('{event_type}', {{
                        key: '{js_key}',
                        code: '{js_code}',
                        keyCode: {key_code},
                        which: {key_code},
                        ctrlKey: {ctrl_key},
                        metaKey: {meta_key},
                        shiftKey: {shift_key},
                        altKey: {alt_key},
                        bubbles: true,
                        cancelable: true
                    }});
                    var activeEl = document.activeElement || document.body;
                    activeEl.dispatchEvent(event);
                    return true;
                }})()"
        )
    }
}

/// JS dispatching a synthetic printable-character `KeyboardEvent` plus the input-value mutation
/// and Ctrl/Meta+A select-all an untrusted event can't drive itself.
pub fn build_regular_key_script(
    key: &str,
    code: &str,
    is_down: bool,
    modifiers: &ModifierState,
) -> String {
    let ch = key.chars().next().unwrap_or(' ');
    build_printable_key_script(key, code, ch as u32, is_down, modifiers)
}

/// [`build_regular_key_script`] with an explicit `keyCode`, for printable special keys whose
/// legacy code differs from the character's (e.g. Numpad5 is 101, not 53).
pub fn build_printable_key_script(
    key: &str,
    code: &str,
    key_code: u32,
    is_down: bool,
    modifiers: &ModifierState,
) -> String {
    let ch = key.chars().next().unwrap_or(' ');
    let event_type = if is_down { "keydown" } else { "keyup" };

    let escaped_key = key.replace('\\', "\\\\").replace('\'', "\\'");
    let escaped_code = code.replace('\\', "\\\\").replace('\'', "\\'");

    let ctrl_key = modifiers.ctrl;
    let meta_key = modifiers.meta;
    let shift_key = modifiers.shift;
    let alt_key = modifiers.alt;

    let is_select_all = is_down && (ch == 'a' || ch == 'A') && (ctrl_key || meta_key);
    let insert_text = insert_text_js("activeEl", &format!("'{escaped_key}'"));

    if is_select_all {
        format!(
            r"(function() {{
                    var activeEl = document.activeElement || document.body;

                    var keydownEvent = new KeyboardEvent('keydown', {{
                        key: '{escaped_key}',
                        code: '{escaped_code}',
                        keyCode: {key_code},
                        which: {key_code},
                        ctrlKey: {ctrl_key},
                        metaKey: {meta_key},
                        shiftKey: {shift_key},
                        altKey: {alt_key},
                        bubbles: true,
                        cancelable: true
                    }});
                    activeEl.dispatchEvent(keydownEvent);

                    if (activeEl.tagName === 'INPUT' || activeEl.tagName === 'TEXTAREA') {{
                        activeEl.select();
                    }} else {{
                        document.execCommand('selectAll', false, null);
                    }}

                    return true;
                }})()"
        )
    } else if is_down {
        format!(
            r"(function() {{
                    var activeEl = document.activeElement || document.body;

                    var keydownEvent = new KeyboardEvent('keydown', {{
                        key: '{escaped_key}',
                        code: '{escaped_code}',
                        keyCode: {key_code},
                        which: {key_code},
                        ctrlKey: {ctrl_key},
                        metaKey: {meta_key},
                        shiftKey: {shift_key},
                        altKey: {alt_key},
                        bubbles: true,
                        cancelable: true
                    }});
                    activeEl.dispatchEvent(keydownEvent);

                    if (!{ctrl_key} && !{meta_key} && !{alt_key}) {{
                        {insert_text}
                    }}

                    return true;
                }})()"
        )
    } else {
        format!(
            r"(function() {{
                    var activeEl = document.activeElement || document.body;
                    var event = new KeyboardEvent('{event_type}', {{
                        key: '{escaped_key}',
                        code: '{escaped_code}',
                        keyCode: {key_code},
                        which: {key_code},
                        ctrlKey: {ctrl_key},
                        metaKey: {meta_key},
                        shiftKey: {shift_key},
                        altKey: {alt_key},
                        bubbles: true,
                        cancelable: true
                    }});
                    activeEl.dispatchEvent(event);
                    return true;
                }})()"
        )
    }
}

/// JS that focuses an element and appends `text` (inputs/textareas/contenteditable).
pub fn build_send_keys_script(js_var: &str, text: &str) -> String {
    let escaped = text
        .replace('\\', "\\\\")
        .replace('`', "\\`")
        .replace('$', "\\$");
    format!(
        r"(function() {{
                var el = window.{js_var};
                if (!el || !el.isConnected) {{
                    throw new Error('stale element reference');
                }}
                el.focus();

                if (el.tagName === 'INPUT' || el.tagName === 'TEXTAREA') {{
                    var nativeInputValueSetter = Object.getOwnPropertyDescriptor(
                        el.tagName === 'INPUT' ? window.HTMLInputElement.prototype : window.HTMLTextAreaElement.prototype,
                        'value'
                    ).set;

                    var newValue = el.value + `{escaped}`;
                    nativeInputValueSetter.call(el, newValue);

                    var inputEvent = new InputEvent('input', {{
                        bubbles: true,
                        cancelable: true,
                        inputType: 'insertText',
                        data: `{escaped}`
                    }});
                    el.dispatchEvent(inputEvent);

                    var changeEvent = new Event('change', {{ bubbles: true }});
                    el.dispatchEvent(changeEvent);
                }} else if (el.isContentEditable) {{
                    document.execCommand('insertText', false, `{escaped}`);
                }}
                return true;
            }})()"
    )
}

/// One segment of an element send-keys request: a run of printable text, or a single special key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeySegment {
    Text(String),
    Special(String),
}

/// Split send-keys text into printable runs and individual special keys, preserving order.
pub fn segment_send_keys(text: &str) -> Vec<KeySegment> {
    let mut segments = Vec::new();
    let mut run = String::new();
    for ch in text.chars() {
        if map_special_key(&ch.to_string()).is_some() {
            if !run.is_empty() {
                segments.push(KeySegment::Text(std::mem::take(&mut run)));
            }
            segments.push(KeySegment::Special(ch.to_string()));
        } else {
            run.push(ch);
        }
    }
    if !run.is_empty() {
        segments.push(KeySegment::Text(run));
    }
    segments
}

/// The modifier keys held down. Each key is tracked on its own, so releasing right Control leaves
/// Control set while left Control is still held.
#[derive(Debug, Clone, Default)]
pub struct HeldModifiers {
    keys: BTreeSet<String>,
}

impl HeldModifiers {
    /// Record a key press; returns whether it newly held a modifier.
    pub fn press(&mut self, key: &str) -> bool {
        is_modifier_key(key) && self.keys.insert(key.to_string())
    }

    /// Record a key release; returns whether it released a held modifier.
    pub fn release(&mut self, key: &str) -> bool {
        self.keys.remove(key)
    }

    /// The modifier flags for the keys still held.
    pub fn state(&self) -> ModifierState {
        let mut state = ModifierState::default();
        for key in &self.keys {
            match map_special_key(key) {
                Some(("Control", _, _)) => state.ctrl = true,
                Some(("Shift", _, _)) => state.shift = true,
                Some(("Alt", _, _)) => state.alt = true,
                Some(("Meta", _, _)) => state.meta = true,
                _ => {}
            }
        }
        state
    }

    /// The held modifier keys in code-point order.
    pub fn keys(&self) -> Vec<String> {
        self.keys.iter().cloned().collect()
    }
}

/// One step of an element send-keys request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SendKeysStep {
    /// Printable text inserted into the element.
    Text(String),
    /// A key event for the focused element, dispatched as the actions path dispatches it.
    Key {
        value: String,
        is_down: bool,
        modifiers: ModifierState,
    },
}

/// Turn send-keys text into ordered steps. A modifier key stays held until the null key or the end
/// of the text (W3C Element Send Keys). While Control, Alt or Meta is held, text is pressed key by
/// key as shortcuts instead of being typed; Shift alone doesn't change the typed text.
pub fn plan_send_keys(text: &str) -> Vec<SendKeysStep> {
    let mut steps = Vec::new();
    let mut held = HeldModifiers::default();

    for segment in segment_send_keys(text) {
        let modifiers = held.state();
        match segment {
            KeySegment::Text(run) if modifiers.ctrl || modifiers.alt || modifiers.meta => {
                for ch in run.chars() {
                    push_key_press(&mut steps, &ch.to_string(), modifiers);
                }
            }
            KeySegment::Text(run) => push_text(&mut steps, &run),
            KeySegment::Special(key) if key == NULL_KEY => {
                release_held_modifiers(&mut steps, &mut held);
            }
            KeySegment::Special(key) if is_modifier_key(&key) => {
                if held.press(&key) {
                    steps.push(SendKeysStep::Key {
                        value: key,
                        is_down: true,
                        modifiers: held.state(),
                    });
                }
            }
            KeySegment::Special(key) => push_key_press(&mut steps, &key, modifiers),
        }
    }
    release_held_modifiers(&mut steps, &mut held);
    steps
}

/// The text to type when a plan presses no keys, so it can go in one step.
pub fn plain_text(steps: &[SendKeysStep]) -> Option<&str> {
    match steps {
        [] => Some(""),
        [SendKeysStep::Text(text)] => Some(text),
        _ => None,
    }
}

fn push_text(steps: &mut Vec<SendKeysStep>, run: &str) {
    if let Some(SendKeysStep::Text(last)) = steps.last_mut() {
        last.push_str(run);
    } else {
        steps.push(SendKeysStep::Text(run.to_string()));
    }
}

fn push_key_press(steps: &mut Vec<SendKeysStep>, value: &str, modifiers: ModifierState) {
    for is_down in [true, false] {
        steps.push(SendKeysStep::Key {
            value: value.to_string(),
            is_down,
            modifiers,
        });
    }
}

/// Key-up every held modifier in code-point order, as the spec's "clear the modifier key state"
/// releases them. Each key-up carries the flags left after its own release.
fn release_held_modifiers(steps: &mut Vec<SendKeysStep>, held: &mut HeldModifiers) {
    for key in held.keys() {
        held.release(&key);
        steps.push(SendKeysStep::Key {
            value: key,
            is_down: false,
            modifiers: held.state(),
        });
    }
}

/// JS that focuses the stored element and puts the caret at the end, so typing continues after the
/// existing content. A contenteditable that already had focus keeps its caret. Throws if the
/// element is stale; runs before any keystroke commits.
pub fn build_send_keys_focus_script(js_var: &str) -> String {
    format!(
        r"(function() {{
                var el = window.{js_var};
                if (!el || !el.isConnected) {{
                    throw new Error('stale element reference');
                }}
                var wasFocused = document.activeElement === el;
                el.focus();
                try {{
                    if (typeof el.setSelectionRange === 'function') {{
                        var end = (el.value || '').length;
                        el.setSelectionRange(end, end);
                    }} else if (el.isContentEditable && !wasFocused) {{
                        var range = document.createRange();
                        range.selectNodeContents(el);
                        range.collapse(false);
                        var selection = window.getSelection();
                        selection.removeAllRanges();
                        selection.addRange(range);
                    }}
                }} catch (e) {{}}
                return true;
            }})()"
    )
}

/// JS that best-effort refocuses the stored element before a key, since a page handler may have
/// moved focus. A focus handler that redirects focus wins: the key follows focus.
pub fn build_refocus_script(js_var: &str) -> String {
    format!(
        r"(function() {{
                var el = window.{js_var};
                if (el && el.isConnected) {{
                    el.focus();
                }}
                return true;
            }})()"
    )
}

/// JS that inserts a printable run into the stored element at its selection, replacing any selected
/// range, and fires `input`. Best-effort: it skips silently when the element is gone — earlier
/// keystrokes in the same request already committed, so throwing here would misreport a partial
/// success — and fires no `change`; the caller dispatches that once after the whole sequence.
pub fn build_insert_run_script(js_var: &str, run: &str) -> String {
    let escaped = run
        .replace('\\', "\\\\")
        .replace('`', "\\`")
        .replace('$', "\\$");
    let insert_text = insert_text_js("el", &format!("`{escaped}`"));
    format!(
        r"(function() {{
                var el = window.{js_var};
                if (!el || !el.isConnected) {{
                    return true;
                }}
                el.focus();
                {insert_text}
                return true;
            }})()"
    )
}

/// JS statements inserting the string expression `text_js` into `el` at its selection, replacing
/// any selected range, then firing `input`. Inputs without a selection API (e.g. `email`) append.
fn insert_text_js(el: &str, text_js: &str) -> String {
    format!(
        r"var insertedText = {text_js};
                if ({el}.tagName === 'INPUT' || {el}.tagName === 'TEXTAREA') {{
                    var nativeInputValueSetter = Object.getOwnPropertyDescriptor(
                        {el}.tagName === 'INPUT' ? window.HTMLInputElement.prototype : window.HTMLTextAreaElement.prototype,
                        'value'
                    ).set;
                    var currentValue = {el}.value;
                    var selStart = null;
                    var selEnd = null;
                    try {{
                        selStart = {el}.selectionStart;
                        selEnd = {el}.selectionEnd;
                    }} catch (e) {{}}
                    if (typeof selStart === 'number' && typeof selEnd === 'number') {{
                        nativeInputValueSetter.call(
                            {el},
                            currentValue.slice(0, selStart) + insertedText + currentValue.slice(selEnd)
                        );
                        var caret = selStart + insertedText.length;
                        {el}.setSelectionRange(caret, caret);
                    }} else {{
                        nativeInputValueSetter.call({el}, currentValue + insertedText);
                    }}
                    {el}.dispatchEvent(new InputEvent('input', {{
                        bubbles: true,
                        cancelable: true,
                        inputType: 'insertText',
                        data: insertedText
                    }}));
                }} else if ({el}.isContentEditable) {{
                    document.execCommand('insertText', false, insertedText);
                }}"
    )
}

/// JS firing the single trailing `change` a split send-keys sequence owes an input/textarea.
/// Best-effort: skips if the element detached mid-sequence, because the keystrokes already
/// committed and a stale error would make a caller retry duplicate the typing.
pub fn build_send_keys_change_script(js_var: &str) -> String {
    format!(
        r"(function() {{
                var el = window.{js_var};
                if (el && el.isConnected && (el.tagName === 'INPUT' || el.tagName === 'TEXTAREA')) {{
                    el.dispatchEvent(new Event('change', {{ bubbles: true }}));
                }}
                return true;
            }})()"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const ESCAPE: &str = "\u{E00C}";
    const ENTER: &str = "\u{E007}";
    const BACKSPACE: &str = "\u{E003}";
    const PAGE_DOWN: &str = "\u{E00F}";
    const ARROW_DOWN: &str = "\u{E015}";
    const SHIFT: &str = "\u{E008}";
    const CONTROL: &str = "\u{E009}";
    const CONTROL_RIGHT: &str = "\u{E051}";
    const ALT: &str = "\u{E00A}";

    fn special_key_range() -> impl Iterator<Item = char> {
        '\u{E000}'..='\u{E05D}'
    }

    fn mods(ctrl: bool, shift: bool, alt: bool, meta: bool) -> ModifierState {
        ModifierState {
            ctrl,
            shift,
            alt,
            meta,
        }
    }

    fn key(value: &str, is_down: bool, modifiers: ModifierState) -> SendKeysStep {
        SendKeysStep::Key {
            value: value.to_string(),
            is_down,
            modifiers,
        }
    }

    fn text(run: &str) -> SendKeysStep {
        SendKeysStep::Text(run.to_string())
    }

    #[test]
    fn every_special_key_code_point_maps() {
        for ch in special_key_range() {
            assert!(
                map_special_key(&ch.to_string()).is_some(),
                "U+{:04X} has no mapping",
                ch as u32
            );
        }
    }

    #[test]
    fn no_mapping_leaks_a_private_use_character() {
        for ch in special_key_range() {
            let (key, code, _) = map_special_key(&ch.to_string()).unwrap();
            assert!(
                !key.chars()
                    .chain(code.chars())
                    .any(|c| ('\u{E000}'..='\u{E05D}').contains(&c)),
                "U+{:04X} maps to a private-use value",
                ch as u32
            );
        }
    }

    #[test]
    fn map_special_key_maps_the_reported_navigation_keys() {
        assert_eq!(map_special_key("\u{E00E}"), Some(("PageUp", "PageUp", 33)));
        assert_eq!(
            map_special_key(PAGE_DOWN),
            Some(("PageDown", "PageDown", 34))
        );
        assert_eq!(map_special_key("\u{E010}"), Some(("End", "End", 35)));
        assert_eq!(map_special_key("\u{E011}"), Some(("Home", "Home", 36)));
        assert_eq!(map_special_key("\u{E016}"), Some(("Insert", "Insert", 45)));
    }

    #[test]
    fn map_special_key_keeps_the_existing_entries() {
        assert_eq!(map_special_key(ESCAPE), Some(("Escape", "Escape", 27)));
        assert_eq!(map_special_key(ENTER), Some(("Enter", "Enter", 13)));
        assert_eq!(
            map_special_key("\u{E006}"),
            Some(("Enter", "NumpadEnter", 13))
        );
        assert_eq!(map_special_key("\u{E00D}"), Some((" ", "Space", 32)));
        assert_eq!(
            map_special_key(ARROW_DOWN),
            Some(("ArrowDown", "ArrowDown", 40))
        );
        assert_eq!(map_special_key("\u{E03C}"), Some(("F12", "F12", 123)));
        assert_eq!(
            map_special_key(CONTROL),
            Some(("Control", "ControlLeft", 17))
        );
    }

    #[test]
    fn map_special_key_maps_numpad_and_right_hand_keys() {
        assert_eq!(map_special_key("\u{E01A}"), Some(("0", "Numpad0", 96)));
        assert_eq!(map_special_key("\u{E023}"), Some(("9", "Numpad9", 105)));
        assert_eq!(
            map_special_key("\u{E029}"),
            Some(("/", "NumpadDivide", 111))
        );
        assert_eq!(map_special_key("\u{E019}"), Some(("=", "NumpadEqual", 187)));
        assert_eq!(map_special_key("\u{E018}"), Some((";", "Semicolon", 186)));
        assert_eq!(
            map_special_key("\u{E050}"),
            Some(("Shift", "ShiftRight", 16))
        );
        assert_eq!(map_special_key("\u{E053}"), Some(("Meta", "MetaRight", 92)));
        assert_eq!(map_special_key("\u{E054}"), Some(("PageUp", "Numpad9", 33)));
        assert_eq!(
            map_special_key("\u{E05D}"),
            Some(("Delete", "NumpadDecimal", 46))
        );
    }

    #[test]
    fn map_special_key_maps_the_remaining_named_keys() {
        assert_eq!(map_special_key(NULL_KEY), Some(("Unidentified", "", 0)));
        assert_eq!(map_special_key("\u{E001}"), Some(("Cancel", "", 3)));
        assert_eq!(map_special_key("\u{E002}"), Some(("Help", "Help", 47)));
        assert_eq!(map_special_key("\u{E005}"), Some(("Clear", "", 12)));
        assert_eq!(map_special_key("\u{E00B}"), Some(("Pause", "Pause", 19)));
        assert_eq!(map_special_key("\u{E040}"), Some(("ZenkakuHankaku", "", 0)));
    }

    #[test]
    fn unassigned_code_points_map_to_unidentified() {
        for unassigned in ["\u{E02A}", "\u{E030}", "\u{E03E}", "\u{E041}", "\u{E04F}"] {
            assert_eq!(map_special_key(unassigned), Some(("Unidentified", "", 0)));
        }
    }

    #[test]
    fn map_special_key_rejects_everything_outside_the_range() {
        assert_eq!(map_special_key("a"), None);
        assert_eq!(map_special_key("5"), None);
        assert_eq!(map_special_key(""), None);
        assert_eq!(map_special_key("\u{D7FF}"), None);
        assert_eq!(map_special_key("\u{E05E}"), None);
        assert_eq!(map_special_key("\u{F8FF}"), None);
        assert_eq!(map_special_key(&format!("{PAGE_DOWN}{PAGE_DOWN}")), None);
    }

    #[test]
    fn printable_special_keys_are_single_characters() {
        let printable: Vec<char> = special_key_range()
            .filter(|ch| is_printable_key(map_special_key(&ch.to_string()).unwrap().0))
            .collect();
        let expected: Vec<char> = std::iter::once('\u{E00D}')
            .chain('\u{E018}'..='\u{E029}')
            .collect();
        assert_eq!(printable, expected);
    }

    #[test]
    fn is_modifier_key_covers_both_hands() {
        for modifier in [
            SHIFT, CONTROL, ALT, "\u{E03D}", "\u{E050}", "\u{E051}", "\u{E052}", "\u{E053}",
        ] {
            assert!(is_modifier_key(modifier));
        }
        assert!(!is_modifier_key(ENTER));
        assert!(!is_modifier_key("a"));
    }

    #[test]
    fn regular_key_code_for_derives_code() {
        assert_eq!(regular_key_code_for("a"), "KeyA");
        assert_eq!(regular_key_code_for("A"), "KeyA");
        assert_eq!(regular_key_code_for("5"), "Digit5");
        assert_eq!(regular_key_code_for("!"), "!");
    }

    #[test]
    fn build_special_key_script_dispatches_keyboard_event() {
        let none = ModifierState::default();
        let script = build_special_key_script("PageDown", "PageDown", 34, true, &none);
        assert!(script.contains("new KeyboardEvent('keydown'"));
        assert!(script.contains("key: 'PageDown'"));
        assert!(script.contains("keyCode: 34"));
        assert!(script.contains("shiftKey: false"));

        let up = build_special_key_script("PageDown", "PageDown", 34, false, &none);
        assert!(up.contains("new KeyboardEvent('keyup'"));
    }

    #[test]
    fn build_special_key_script_carries_held_modifiers_on_every_event() {
        let shift = mods(false, true, false, false);
        let ctrl_alt_meta = mods(true, false, true, true);
        for script in [
            build_special_key_script("Enter", "Enter", 13, true, &shift),
            build_special_key_script("Enter", "Enter", 13, false, &shift),
            build_special_key_script("Backspace", "Backspace", 8, true, &shift),
            build_special_key_script("ArrowDown", "ArrowDown", 40, true, &shift),
        ] {
            assert!(script.contains("shiftKey: true"), "{script}");
            assert!(script.contains("ctrlKey: false"), "{script}");
        }
        let chord = build_special_key_script("Home", "Home", 36, true, &ctrl_alt_meta);
        assert!(chord.contains("ctrlKey: true"));
        assert!(chord.contains("altKey: true"));
        assert!(chord.contains("metaKey: true"));
        assert!(chord.contains("shiftKey: false"));
    }

    #[test]
    fn build_regular_key_script_carries_modifiers_and_value_mutation() {
        let script = build_regular_key_script("a", "KeyA", true, &mods(false, false, false, false));
        assert!(script.contains("key: 'a'"));
        assert!(script.contains("keyCode: 97"));
        assert!(script.contains("inputType: 'insertText'"));

        let select_all =
            build_regular_key_script("a", "KeyA", true, &mods(true, false, false, false));
        assert!(select_all.contains("activeEl.select()"));
    }

    #[test]
    fn build_printable_key_script_uses_the_given_key_code() {
        let script = build_printable_key_script(
            "5",
            "Numpad5",
            101,
            true,
            &mods(false, false, false, false),
        );
        assert!(script.contains("key: '5'"));
        assert!(script.contains("code: 'Numpad5'"));
        assert!(script.contains("keyCode: 101"));
        assert!(script.contains("var insertedText = '5'"));
    }

    #[test]
    fn build_printable_key_script_types_into_contenteditable() {
        let script =
            build_printable_key_script(" ", "Space", 32, true, &mods(false, false, false, false));
        assert!(script.contains("activeEl.isContentEditable"));
        assert!(script.contains("document.execCommand('insertText', false, insertedText)"));
    }

    #[test]
    fn typed_text_replaces_the_selection() {
        let none = mods(false, false, false, false);
        for script in [
            build_printable_key_script("5", "Numpad5", 101, true, &none),
            build_insert_run_script("__wd_el_abc", "new"),
        ] {
            assert!(script.contains(
                "currentValue.slice(0, selStart) + insertedText + currentValue.slice(selEnd)"
            ));
            assert!(script.contains("setSelectionRange(caret, caret)"));
        }
    }

    #[test]
    fn build_send_keys_script_focuses_and_inserts() {
        let script = build_send_keys_script("__wd_el_abc", "hi");
        assert!(script.contains("window.__wd_el_abc"));
        assert!(script.contains("el.focus()"));
        assert!(script.contains("`hi`"));
        assert!(script.contains("new Event('change'"));
    }

    #[test]
    fn segment_send_keys_splits_text_and_special_keys() {
        assert_eq!(
            segment_send_keys("hello"),
            vec![KeySegment::Text("hello".into())]
        );
        assert_eq!(segment_send_keys(""), vec![]);
        assert_eq!(
            segment_send_keys(ESCAPE),
            vec![KeySegment::Special(ESCAPE.into())]
        );

        assert_eq!(
            segment_send_keys(&format!("hi{ENTER}")),
            vec![
                KeySegment::Text("hi".into()),
                KeySegment::Special(ENTER.into())
            ]
        );

        assert_eq!(
            segment_send_keys(&format!("a{ARROW_DOWN}bc")),
            vec![
                KeySegment::Text("a".into()),
                KeySegment::Special(ARROW_DOWN.into()),
                KeySegment::Text("bc".into()),
            ]
        );

        assert_eq!(
            segment_send_keys(&format!("{ESCAPE}{ENTER}")),
            vec![
                KeySegment::Special(ESCAPE.into()),
                KeySegment::Special(ENTER.into())
            ]
        );
    }

    #[test]
    fn segment_send_keys_never_leaves_a_special_key_in_text() {
        let everything: String = special_key_range().flat_map(|ch| ['x', ch]).collect();
        for segment in segment_send_keys(&everything) {
            if let KeySegment::Text(run) = segment {
                assert_eq!(run, "x");
            }
        }
    }

    #[test]
    fn plan_send_keys_types_plain_text_in_one_step() {
        assert_eq!(plan_send_keys("hello"), vec![text("hello")]);
        assert_eq!(plan_send_keys(""), vec![]);
    }

    #[test]
    fn plain_text_drops_null_keys() {
        assert_eq!(plan_send_keys(&format!("a{NULL_KEY}b")), vec![text("ab")]);
        assert_eq!(
            plain_text(&plan_send_keys(&format!("a{NULL_KEY}b"))),
            Some("ab")
        );
        assert_eq!(plain_text(&plan_send_keys(NULL_KEY)), Some(""));
        assert_eq!(plain_text(&plan_send_keys("")), Some(""));
        assert_eq!(plain_text(&plan_send_keys(&format!("hi{ENTER}"))), None);
    }

    #[test]
    fn held_modifiers_track_each_hand() {
        let mut held = HeldModifiers::default();
        assert!(held.press(CONTROL));
        assert!(held.press(CONTROL_RIGHT));
        assert!(!held.press(CONTROL));
        assert!(!held.press("a"));
        assert!(held.release(CONTROL_RIGHT));
        assert_eq!(held.state(), mods(true, false, false, false));
        assert!(held.release(CONTROL));
        assert_eq!(held.state(), ModifierState::default());
        assert!(!held.release(CONTROL));
    }

    #[test]
    fn held_modifiers_map_every_modifier_key() {
        let mut held = HeldModifiers::default();
        for key in ["\u{E050}", "\u{E052}", "\u{E053}"] {
            held.press(key);
        }
        assert_eq!(held.state(), mods(false, true, true, true));
        assert_eq!(held.keys(), vec!["\u{E050}", "\u{E052}", "\u{E053}"]);
    }

    #[test]
    fn plan_send_keys_presses_special_keys_between_text_runs() {
        let none = ModifierState::default();
        assert_eq!(
            plan_send_keys(&format!("abc{BACKSPACE}d{PAGE_DOWN}")),
            vec![
                text("abc"),
                key(BACKSPACE, true, none),
                key(BACKSPACE, false, none),
                text("d"),
                key(PAGE_DOWN, true, none),
                key(PAGE_DOWN, false, none),
            ]
        );
    }

    #[test]
    fn plan_send_keys_holds_modifiers_until_the_end() {
        let none = ModifierState::default();
        let shift = mods(false, true, false, false);
        assert_eq!(
            plan_send_keys(&format!("{SHIFT}ab{ENTER}")),
            vec![
                key(SHIFT, true, shift),
                text("ab"),
                key(ENTER, true, shift),
                key(ENTER, false, shift),
                key(SHIFT, false, none),
            ]
        );
    }

    #[test]
    fn plan_send_keys_presses_text_as_shortcuts_under_control() {
        let none = ModifierState::default();
        let ctrl = mods(true, false, false, false);
        assert_eq!(
            plan_send_keys(&format!("{CONTROL}a{BACKSPACE}")),
            vec![
                key(CONTROL, true, ctrl),
                key("a", true, ctrl),
                key("a", false, ctrl),
                key(BACKSPACE, true, ctrl),
                key(BACKSPACE, false, ctrl),
                key(CONTROL, false, none),
            ]
        );
    }

    #[test]
    fn plan_send_keys_null_key_releases_modifiers() {
        let none = ModifierState::default();
        let ctrl = mods(true, false, false, false);
        assert_eq!(
            plan_send_keys(&format!("{CONTROL}a{NULL_KEY}b")),
            vec![
                key(CONTROL, true, ctrl),
                key("a", true, ctrl),
                key("a", false, ctrl),
                key(CONTROL, false, none),
                text("b"),
            ]
        );
    }

    #[test]
    fn plan_send_keys_ignores_a_repeated_modifier_and_releases_in_code_point_order() {
        let none = ModifierState::default();
        let alt = mods(false, false, true, false);
        let alt_shift = mods(false, true, true, false);
        assert_eq!(
            plan_send_keys(&format!("{ALT}{SHIFT}{ALT}")),
            vec![
                key(ALT, true, alt),
                key(SHIFT, true, alt_shift),
                key(SHIFT, false, alt),
                key(ALT, false, none),
            ]
        );
    }

    #[test]
    fn plan_send_keys_keeps_control_while_the_other_hand_holds_it() {
        let ctrl = mods(true, false, false, false);
        let steps = plan_send_keys(&format!("{CONTROL}{CONTROL_RIGHT}a"));
        assert_eq!(steps[2], key("a", true, ctrl));
        assert_eq!(
            steps[4..],
            [
                key(CONTROL, false, ctrl),
                key(CONTROL_RIGHT, false, ModifierState::default()),
            ]
        );
    }

    #[test]
    fn build_send_keys_focus_script_guards_stale_and_moves_caret() {
        let script = build_send_keys_focus_script("__wd_el_abc");
        assert!(script.contains("throw new Error('stale element reference')"));
        assert!(script.contains("el.focus()"));
        assert!(script.contains("setSelectionRange(end, end)"));
        assert!(script.contains("el.isContentEditable && !wasFocused"));
        assert!(script.contains("range.collapse(false)"));
    }

    #[test]
    fn build_insert_run_script_inserts_fires_input_and_defers_change() {
        let script = build_insert_run_script("__wd_el_abc", "hi");
        assert!(script.contains("window.__wd_el_abc"));
        assert!(script.contains("var insertedText = `hi`"));
        assert!(script.contains("inputType: 'insertText'"));
        assert!(script.contains("document.execCommand('insertText', false, insertedText)"));
        assert!(!script.contains("new Event('change'"));
    }

    #[test]
    fn build_insert_run_script_escapes_backtick_dollar_and_backslash() {
        let script = build_insert_run_script("v", "a`$\\b");
        assert!(script.contains(r"a\`\$\\b"));
    }

    #[test]
    fn build_send_keys_change_script_fires_change_without_throwing() {
        let script = build_send_keys_change_script("__wd_el_abc");
        assert!(script.contains("new Event('change'"));
        assert!(!script.contains("throw"));
    }
}
