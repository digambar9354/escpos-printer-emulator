use crate::emulator::EmulatorState;
use egui::{Align2, Color32, RichText, Ui};
use std::process::Command;
use std::sync::mpsc::{channel, Receiver};
use std::thread;

/// Result of a printer/network operation, shown to the user in a dialog.
#[derive(Clone)]
struct OpResult {
    title: String,
    body: String,
    ok: bool,
}

pub struct SettingsPanel {
    /// The dialog currently shown (None = no dialog).
    result: Option<OpResult>,
    /// Receiver for a background operation in progress.
    pending: Option<Receiver<OpResult>>,
}

impl Default for SettingsPanel {
    fn default() -> Self {
        Self {
            result: None,
            pending: None,
        }
    }
}

impl SettingsPanel {
    pub fn show(&mut self, ui: &mut Ui, _state: &mut EmulatorState) {
        // Collect the result of any finished background operation.
        if let Some(rx) = &self.pending {
            if let Ok(res) = rx.try_recv() {
                self.result = Some(res);
                self.pending = None;
            }
        }
        let busy = self.pending.is_some();

        ui.heading("Emulator Settings");
        ui.separator();
        ui.add_space(4.0);

        ui.add_enabled_ui(!busy, |ui| {
            // Virtual printer management
            ui.group(|ui| {
                ui.label(RichText::new("Virtual Printer Management").strong());
                ui.label("Install the emulator as a system printer.");
                ui.add_space(4.0);

                ui.horizontal_wrapped(|ui| {
                    if ui.button("🖨 Install Windows Printer").clicked() {
                        self.run("Install Windows Printer", install_windows_printer);
                    }
                    if ui.button("🐧 Install Linux Printer").clicked() {
                        self.run("Install Linux Printer", install_linux_printer);
                    }
                    if ui.button("🗑 Uninstall Printer").clicked() {
                        self.run("Uninstall Printer", uninstall_printer);
                    }
                    if ui.button("🔍 Check Status").clicked() {
                        self.run("Printer Status", check_printer_status);
                    }
                });

                ui.add_space(2.0);
                ui.label(
                    RichText::new("Note: installing/uninstalling requires administrator privileges.")
                        .small()
                        .weak(),
                );
            });

            ui.add_space(6.0);

            // Network settings
            ui.group(|ui| {
                ui.label(RichText::new("Network Configuration").strong());
                ui.label("Listening on 0.0.0.0:9100 (reachable from the local network).");
                ui.add_space(4.0);
                if ui.button("📡 Test Connection").clicked() {
                    self.run("Test Connection", test_network_connection);
                }
            });

            ui.add_space(6.0);

            // Information about operation
            ui.group(|ui| {
                ui.label(RichText::new("Automatic Operation").strong());
                ui.label("• Respects ESC/POS standards automatically");
                ui.label("• Paper width: 50mm, 78mm, 80mm (auto-detection)");
                ui.label("• Font, justification, emphasis via ESC/POS commands");
                ui.label("• No manual configuration needed");
            });
        });

        self.show_result_dialog(ui);
    }

    /// Run an operation on a background thread and show a progress/result dialog.
    fn run<F>(&mut self, title: &str, op: F)
    where
        F: FnOnce() -> OpResult + Send + 'static,
    {
        let (tx, rx) = channel();
        thread::spawn(move || {
            let _ = tx.send(op());
        });
        self.pending = Some(rx);
        self.result = Some(OpResult {
            title: title.to_string(),
            body: "Working…".to_string(),
            ok: true,
        });
    }

    fn show_result_dialog(&mut self, ui: &mut Ui) {
        let busy = self.pending.is_some();
        let mut close = false;

        if let Some(res) = &self.result {
            let title = res.title.clone();
            let body = res.body.clone();
            let ok = res.ok;
            let mut open = true;

            egui::Window::new(title)
                .collapsible(false)
                .resizable(false)
                .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
                .open(&mut open)
                .show(ui.ctx(), |ui| {
                    ui.set_min_width(300.0);
                    if busy {
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label("Working…");
                        });
                    } else {
                        let color = if ok {
                            Color32::from_rgb(64, 200, 120)
                        } else {
                            Color32::from_rgb(240, 130, 120)
                        };
                        ui.label(RichText::new(body).monospace().color(color));
                        ui.add_space(10.0);
                        if ui.button("OK").clicked() {
                            close = true;
                        }
                    }
                });

            // Allow closing via the window's X only when not busy.
            if !open && !busy {
                close = true;
            }
        }

        if close {
            self.result = None;
        }
    }
}

/// Run a PowerShell command and return (success, combined output).
fn powershell(command: &str) -> (bool, String) {
    match Command::new("powershell")
        .args(["-NoProfile", "-Command", command])
        .output()
    {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            let body = if !stderr.is_empty() { stderr } else { stdout };
            (output.status.success(), body)
        }
        Err(e) => (false, format!("Cannot run PowerShell: {}", e)),
    }
}

fn install_windows_printer() -> OpResult {
    let (ok, body) = powershell(
        "Add-PrinterPort -Name '127.0.0.1:9100' -PrinterHostAddress '127.0.0.1' -PortNumber 9100 -ErrorAction SilentlyContinue; \
         $driver = (Get-PrinterDriver | Where-Object { $_.Name -like '*Microsoft*' } | Select-Object -First 1).Name; \
         Add-Printer -Name 'ESC_POS_Virtual_Printer' -DriverName $driver -PortName '127.0.0.1:9100'; \
         Write-Host 'Printer installed successfully'",
    );
    let body = if body.is_empty() {
        if ok { "Printer installed successfully.".to_string() } else { "Installation failed. Try running as Administrator.".to_string() }
    } else {
        body
    };
    OpResult { title: "Install Windows Printer".to_string(), body, ok }
}

fn uninstall_printer() -> OpResult {
    let (ok, body) = powershell(
        "Remove-Printer -Name 'ESC_POS_Virtual_Printer' -Confirm:$false -ErrorAction SilentlyContinue; \
         Remove-PrinterPort -Name '127.0.0.1:9100' -ErrorAction SilentlyContinue; \
         Write-Host 'Printer uninstalled successfully'",
    );
    let body = if body.is_empty() {
        "Printer uninstalled.".to_string()
    } else {
        body
    };
    OpResult { title: "Uninstall Printer".to_string(), body, ok }
}

fn check_printer_status() -> OpResult {
    let (_ok, body) = powershell(
        "Get-Printer -Name 'ESC_POS_Virtual_Printer' -ErrorAction SilentlyContinue | \
         Format-List Name, PortName, DriverName, PrinterStatus",
    );
    if body.trim().is_empty() {
        OpResult {
            title: "Printer Status".to_string(),
            body: "Virtual printer is NOT installed.".to_string(),
            ok: false,
        }
    } else {
        OpResult {
            title: "Printer Status".to_string(),
            body,
            ok: true,
        }
    }
}

fn test_network_connection() -> OpResult {
    let (_ok, body) = powershell(
        "(Test-NetConnection -ComputerName 127.0.0.1 -Port 9100 -WarningAction SilentlyContinue).TcpTestSucceeded",
    );
    let success = body.contains("True");
    let body = if success {
        "✅ Port 9100 is reachable — the server is listening.".to_string()
    } else {
        "❌ Port 9100 is not reachable.\nMake sure the emulator is running.".to_string()
    };
    OpResult { title: "Test Connection".to_string(), body, ok: success }
}

fn install_linux_printer() -> OpResult {
    let result = Command::new("bash")
        .args([
            "-c",
            "if command -v lpadmin >/dev/null 2>&1; then \
                sudo lpadmin -p ESC_POS_Linux_Printer -E -v socket://127.0.0.1:9100 -m 'Generic Text-Only Printer' && \
                sudo lpadmin -d ESC_POS_Linux_Printer && \
                echo 'Linux printer installed successfully'; \
             else \
                echo 'CUPS (lpadmin) not found. Install CUPS first.'; exit 1; \
             fi",
        ])
        .output();

    match result {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            let body = if !stdout.is_empty() { stdout } else { stderr };
            OpResult {
                title: "Install Linux Printer".to_string(),
                body: if body.is_empty() { "Done.".to_string() } else { body },
                ok: output.status.success(),
            }
        }
        Err(e) => OpResult {
            title: "Install Linux Printer".to_string(),
            body: format!("Cannot run bash/lpadmin: {}\n(This is expected on Windows.)", e),
            ok: false,
        },
    }
}
