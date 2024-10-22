#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
#![allow(dead_code)]
#![allow(unused_imports)]

use std::process::{Child, Command};
use std::str::FromStr;
use std::{alloc, thread};
use std::{path::PathBuf, time::Duration};

use indexmap::IndexSet;
use known_folders::{get_known_folder_path, KnownFolder};
use serde::{Deserialize, Serialize};
use tao::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use tray_icon::{
    menu::{Menu, MenuEvent, MenuEventReceiver, MenuItem, PredefinedMenuItem},
    MouseButton, TrayIcon, TrayIconBuilder, TrayIconEvent, TrayIconEventReceiver,
};

fn main() -> anyhow::Result<()> {
    let event_loop = EventLoop::new();

    let mut window = Some(
        WindowBuilder::new()
            .with_title("A fantastic window!")
            .with_visible(false)
            .build(&event_loop)
            .unwrap(),
    );

    let quit_i = MenuItem::new("Quit", true, None);
    let asdf = MenuItem::new("Asdf", true, None);

    let mut tray_icon = None;
    let mut child: Option<Child> = None;
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::WaitUntil(
            std::time::Instant::now() + std::time::Duration::from_millis(16),
        );

        if let tao::event::Event::NewEvents(tao::event::StartCause::Init) = event {
            let tray_menu = Menu::new();

            tray_menu.append_items(&[&asdf, &quit_i]).unwrap();
            let icon = load_icon(include_bytes!("../assets/icon.png"));

            tray_icon = Some(
                TrayIconBuilder::new()
                    .with_menu_on_left_click(false)
                    .with_menu(Box::new(tray_menu.clone()))
                    .with_tooltip("Workspace")
                    .with_icon(icon)
                    .build()
                    .unwrap(),
            );

            // We have to request a redraw here to have the icon actually show up.
            // Tao only exposes a redraw method on the Window so we use core-foundation directly.
            #[cfg(target_os = "macos")]
            unsafe {
                use core_foundation::runloop::{CFRunLoopGetMain, CFRunLoopWakeUp};

                let rl = CFRunLoopGetMain();
                CFRunLoopWakeUp(rl);
            }
        }

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id: _,
                ..
            } => {
                if let Some(mut child) = child.take() {
                    _ = child.kill();
                }
                // drop the window to fire the `Destroyed` event
                window = None;
            }
            Event::WindowEvent {
                event: WindowEvent::Destroyed,
                window_id: _,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
                _ = tray_icon.take();
            }
            Event::MainEventsCleared => {
                if let Some(w) = &window {
                    w.request_redraw();
                }
            }
            _ => (),
        }
        if let Ok(event) = MenuEvent::receiver().try_recv() {
            if event.id == quit_i.id() {
                window = None;
            } else if event.id == asdf.id() && child.is_none() {
                child = Command::new("game.exe").spawn().ok();
            }
        }
        if let Some(c) = &mut child {
            if let Ok(Some(_)) = c.try_wait() {
                child = None;
            }
        }
    });
}

fn load_icon(img: &[u8]) -> tray_icon::Icon {
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::load_from_memory(img)
            .expect("Failed to open icon path")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };
    tray_icon::Icon::from_rgba(icon_rgba, icon_width, icon_height).expect("Failed to open icon")
}
