//! Shared installed clipboard provider access. Terminal authorization stays with its caller.
use super::*;
impl Runtime {
    pub(crate) fn read_clipboard_text(&mut self) -> Result<String> {
        if let Some(clipboard) = &mut self.clipboard
            && let Ok(text) = clipboard
                .get()
                .clipboard(LinuxClipboardKind::Clipboard)
                .text()
            && !text.is_empty()
        {
            return Ok(text);
        }
        self.read_clipboard_text_fallback()
    }

    pub(crate) fn write_clipboard_text(&mut self, text: &str) -> Result<()> {
        self.write_terminal_clipboard_text(
            datum_gui_protocol::TerminalClipboardSelection::Clipboard,
            text,
        )
    }

    pub(crate) fn write_terminal_clipboard_text(
        &mut self,
        selection: datum_gui_protocol::TerminalClipboardSelection,
        text: &str,
    ) -> Result<()> {
        let kind = match selection {
            datum_gui_protocol::TerminalClipboardSelection::Clipboard => {
                LinuxClipboardKind::Clipboard
            }
            datum_gui_protocol::TerminalClipboardSelection::Primary => LinuxClipboardKind::Primary,
        };
        if let Some(clipboard) = &mut self.clipboard
            && clipboard
                .set()
                .clipboard(kind)
                .text(text.to_string())
                .is_ok()
        {
            return Ok(());
        }
        self.write_clipboard_text_fallback(selection, text)
    }

    pub(crate) fn read_clipboard_text_fallback(&self) -> Result<String> {
        let output = Command::new("/usr/bin/xclip")
            .args(["-selection", "clipboard", "-target", "UTF8_STRING", "-o"])
            .output()
            .context("read clipboard with xclip")?;
        if !output.status.success() {
            anyhow::bail!("xclip clipboard read failed");
        }
        String::from_utf8(output.stdout).context("clipboard text is not UTF-8")
    }

    pub(crate) fn write_clipboard_text_fallback(
        &self,
        selection: datum_gui_protocol::TerminalClipboardSelection,
        text: &str,
    ) -> Result<()> {
        let selection = match selection {
            datum_gui_protocol::TerminalClipboardSelection::Clipboard => "clipboard",
            datum_gui_protocol::TerminalClipboardSelection::Primary => "primary",
        };
        let mut child = Command::new("/usr/bin/xclip")
            .args(["-selection", selection, "-target", "UTF8_STRING"])
            .stdin(Stdio::piped())
            .spawn()
            .context("spawn xclip for clipboard write")?;
        let mut stdin = child.stdin.take().context("take xclip stdin")?;
        stdin
            .write_all(text.as_bytes())
            .context("write clipboard text to xclip")?;
        drop(stdin);
        let status = child.wait().context("wait for xclip clipboard write")?;
        if !status.success() {
            anyhow::bail!("xclip clipboard write failed");
        }
        Ok(())
    }
}
