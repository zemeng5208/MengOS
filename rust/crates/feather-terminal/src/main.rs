//! FeatherTerminal：MengOS 的终端窗口（GTK4 + VTE4）。
//!
//! 本文件只做三件事：开一个窗口、在里面放一个真正的 VTE 终端、用 `spawn_async`
//! 启动真实的登录 shell。没有模拟终端，也没有自己解析命令。

use std::path::PathBuf;

use gtk::prelude::*;
use vte4::prelude::*;

/// 依次尝试的 shell：`/usr/bin/zsh` → 环境变量 `$SHELL` → `/bin/bash`。
fn shell_candidates() -> Vec<String> {
    let mut candidates: Vec<String> = Vec::new();

    if PathBuf::from("/usr/bin/zsh").exists() {
        candidates.push(String::from("/usr/bin/zsh"));
    }

    if let Ok(shell) = std::env::var("SHELL") {
        if !shell.is_empty() && PathBuf::from(&shell).exists() && !candidates.contains(&shell) {
            candidates.push(shell);
        }
    }

    if !candidates.contains(&String::from("/bin/bash")) {
        candidates.push(String::from("/bin/bash"));
    }

    candidates
}

fn main() {
    let application = gtk::Application::new(Some("com.mengos.FeatherTerminal"), Default::default());
    application.connect_activate(build_ui);
    application.run();
}

fn build_ui(application: &gtk::Application) {
    let window = gtk::ApplicationWindow::new(application);
    window.set_title(Some("FeatherTerminal"));
    window.set_default_size(1000, 650);

    let terminal = vte4::Terminal::new();
    terminal.set_vexpand(true);
    terminal.set_hexpand(true);
    window.set_child(Some(&terminal));

    // shell 退出（例如输入 exit）就关闭窗口。
    let window_for_exit = window.clone();
    terminal.connect_child_exited(move |_, _status| {
        window_for_exit.close();
    });

    install_shortcuts(&window, &terminal);
    spawn_shell(&terminal);

    window.present();
}

/// Ctrl+Shift+C 复制、Ctrl+Shift+V 粘贴；Ctrl+C 不拦截，直接交给终端里的程序。
fn install_shortcuts(window: &gtk::ApplicationWindow, terminal: &vte4::Terminal) {
    let controller = gtk::EventControllerKey::new();
    let terminal = terminal.clone();

    controller.connect_key_pressed(move |_, keyval, _keycode, state| {
        let ctrl_shift = gtk::gdk::ModifierType::CONTROL_MASK | gtk::gdk::ModifierType::SHIFT_MASK;
        if !state.contains(ctrl_shift) {
            return gtk::glib::Propagation::Proceed;
        }

        if keyval == gtk::gdk::Key::C {
            terminal.copy_clipboard_format(vte4::Format::Text);
            return gtk::glib::Propagation::Stop;
        }
        if keyval == gtk::gdk::Key::V {
            terminal.paste_clipboard();
            return gtk::glib::Propagation::Stop;
        }

        gtk::glib::Propagation::Proceed
    });

    window.add_controller(controller);
}

/// 用 VTE 的 `spawn_async` 在伪终端里启动真实的 shell，工作目录为用户主目录。
fn spawn_shell(terminal: &vte4::Terminal) {
    let shell = match shell_candidates().into_iter().next() {
        Some(shell) => shell,
        None => {
            eprintln!("找不到可用的 shell");
            return;
        }
    };
    let home = std::env::var("HOME").unwrap_or_else(|_| String::from("/"));

    // VTE 的 envv 参数会替换整个环境，所以显式传当前环境，避免 shell 没有 PATH/HOME。
    let env: Vec<String> = std::env::vars().map(|(k, v)| format!("{k}={v}")).collect();
    let envv: Vec<&str> = env.iter().map(String::as_str).collect();
    let argv = [shell.as_str()];
    let shell_for_log = shell.clone();

    terminal.spawn_async(
        vte4::PtyFlags::DEFAULT,
        Some(home.as_str()),
        &argv,
        &envv,
        gtk::glib::SpawnFlags::DEFAULT,
        || {},
        -1,
        None::<&gtk::gio::Cancellable>,
        move |result| {
            if let Err(error) = result {
                eprintln!("启动 {shell_for_log} 失败：{error}");
            }
        },
    );
}
