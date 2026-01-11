use crate::{AliasEntry, CURRENT_ALIAS, LocalAliasPlugin};
use aviutl2::generic::GenericPlugin;
use eframe::egui;
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use std::ffi::c_void;
use std::sync::{Arc, Mutex};
use windows::Win32::{
    Foundation::{HWND, RECT},
    UI::WindowsAndMessaging::{
        GWL_EXSTYLE, GWL_STYLE, GetClientRect, GetWindowLongPtrW, MoveWindow, SW_SHOW,
        SWP_FRAMECHANGED, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER, SetParent, SetWindowLongPtrW,
        SetWindowPos, ShowWindow, WS_CAPTION, WS_CHILD, WS_EX_CLIENTEDGE, WS_EX_DLGMODALFRAME,
        WS_EX_STATICEDGE, WS_EX_WINDOWEDGE, WS_MAXIMIZEBOX, WS_MINIMIZEBOX, WS_POPUP, WS_SYSMENU,
        WS_THICKFRAME, WS_VISIBLE, WS_BORDER, WS_DLGFRAME,
    },
};

pub(crate) struct UiState {
    pub(crate) aliases: Vec<AliasEntry>,
    pub(crate) selected_index: Option<usize>,
    pub(crate) version: String,
    pub(crate) show_info: bool,
    pub(crate) rename_index: Option<usize>,
    pub(crate) rename_buffer: String,
    pub(crate) confirm_delete_index: Option<usize>,
}

impl UiState {
    pub(crate) fn new() -> Self {
        Self {
            aliases: Vec::new(),
            selected_index: None,
            version: env!("CARGO_PKG_VERSION").to_string(),
            show_info: false,
            rename_index: None,
            rename_buffer: String::new(),
            confirm_delete_index: None,
        }
    }
}

pub(crate) struct LocalAliasUiApp {
    state: Arc<Mutex<UiState>>,
    repaint_ctx: Arc<Mutex<Option<egui::Context>>>,
    parent_hwnd: isize,
    embedded: bool,
    child_hwnd: Option<isize>,
}

impl LocalAliasUiApp {
    pub(crate) fn new(
        state: Arc<Mutex<UiState>>,
        repaint_ctx: Arc<Mutex<Option<egui::Context>>>,
        parent_hwnd: isize,
    ) -> Self {
        Self {
            state,
            repaint_ctx,
            parent_hwnd,
            embedded: false,
            child_hwnd: None,
        }
    }

    fn set_repaint_context(&self, ctx: &egui::Context) {
        if let Ok(mut slot) = self.repaint_ctx.lock()
            && slot.is_none()
        {
            *slot = Some(ctx.clone());
        }
    }

    fn handle_add_alias(&self) {
        let new_alias = LocalAliasPlugin::add_alias_from_focus().ok().flatten();
        if let Some(entry) = new_alias {
            let mut state = self.state.lock().unwrap();
            state.aliases.push(entry);
            sync_aliases_to_plugin(&state);
        }
    }

    fn render_alias_list(&self, ui: &mut egui::Ui) {
        let mut action = None;
        let state = self.state.lock().unwrap();
        if state.aliases.is_empty() {
            ui.label(
                "エイリアスがありません。オブジェクトを選択した後、＋ボタンで追加してください。",
            );
            return;
        }

        for (index, alias) in state.aliases.iter().enumerate() {
            let is_selected = state.selected_index == Some(index);
            egui::Frame::group(ui.style())
                .fill(if is_selected {
                    ui.visuals().selection.bg_fill
                } else {
                    ui.visuals().extreme_bg_color
                })
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        if ui
                            .selectable_label(is_selected, &alias.name)
                            .on_hover_text("選択")
                            .clicked()
                        {
                            action = Some(UiAction::Select(index));
                        }
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui
                                .add_enabled(
                                    index + 1 < state.aliases.len(),
                                    egui::Button::new("▼"),
                                )
                                .on_hover_text("下へ")
                                .clicked()
                            {
                                action = Some(UiAction::Move(index, 1));
                            }
                            if ui
                                .add_enabled(index > 0, egui::Button::new("▲"))
                                .on_hover_text("上へ")
                                .clicked()
                            {
                                action = Some(UiAction::Move(index, -1));
                            }
                            if ui.button("🗑").on_hover_text("削除").clicked() {
                                action = Some(UiAction::ConfirmDelete(index));
                            }
                            if ui.button("✎").on_hover_text("名前変更").clicked() {
                                action = Some(UiAction::StartRename(index));
                            }
                        });
                    });
                });
        }
        drop(state);
        if let Some(action) = action {
            self.apply_action(action);
        }
    }

    fn apply_action(&self, action: UiAction) {
        let mut state = self.state.lock().unwrap();
        match action {
            UiAction::Select(index) => {
                state.selected_index = Some(index);
                sync_current_alias(&state);
            }
            UiAction::StartRename(index) => {
                if let Some(name) = state.aliases.get(index).map(|alias| alias.name.clone()) {
                    state.rename_index = Some(index);
                    state.rename_buffer = name;
                }
            }
            UiAction::ConfirmDelete(index) => {
                if index < state.aliases.len() {
                    state.confirm_delete_index = Some(index);
                }
            }
            UiAction::Move(index, dir) => {
                let new_index = if dir < 0 {
                    index.saturating_sub(1)
                } else {
                    index + 1
                };
                if new_index < state.aliases.len() && new_index != index {
                    let item = state.aliases.remove(index);
                    state.aliases.insert(new_index, item);
                    if state.selected_index == Some(index) {
                        state.selected_index = Some(new_index);
                    }
                    sync_aliases_to_plugin(&state);
                    sync_current_alias(&state);
                }
            }
        }
    }

    fn render_info_modal(&self, ctx: &egui::Context) {
        let mut open = {
            let state = self.state.lock().unwrap();
            state.show_info
        };
        if !open {
            return;
        }

        egui::Window::new("Rusty Local Alias Plugin")
            .collapsible(false)
            .resizable(false)
            .open(&mut open)
            .show(ctx, |ui| {
                let state = self.state.lock().unwrap();
                ui.label(format!("バージョン: {}", state.version));
                ui.add_space(8.0);
                ui.label("プロジェクトローカルなエイリアスを管理するAviUtl2プラグイン。");
                ui.label("ここでエイリアスを選択した後、カスタムオブジェクト「Rusty Local Alias」を配置し、その位置にシークバーを移動させてください。");
                ui.add_space(8.0);
                ui.label("Developed by Nanashi.");
                ui.label("Source: https://github.com/sevenc-nanashi/aviutl2-rs");
            });

        if let Ok(mut state) = self.state.lock() {
            state.show_info = open;
        }
    }

    fn render_rename_modal(&self, ctx: &egui::Context) {
        let (rename_index, buffer) = {
            let state = self.state.lock().unwrap();
            (state.rename_index, state.rename_buffer.clone())
        };
        let Some(index) = rename_index else {
            return;
        };

        let mut open = true;
        let mut new_buffer = buffer;
        let mut submit = false;
        let mut cancel = false;
        egui::Window::new("名前変更")
            .collapsible(false)
            .resizable(false)
            .open(&mut open)
            .show(ctx, |ui| {
                ui.label("新しいエイリアス名");
                let response = ui.text_edit_singleline(&mut new_buffer);
                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    submit = true;
                }
                ui.horizontal(|ui| {
                    if ui.button("OK").clicked() {
                        submit = true;
                    }
                    if ui.button("キャンセル").clicked() {
                        cancel = true;
                    }
                });
            });

        if submit || cancel {
            open = false;
        }

        let mut state = self.state.lock().unwrap();
        if index >= state.aliases.len() {
            state.rename_index = None;
            state.rename_buffer.clear();
            return;
        }

        if !open {
            if submit
                && !new_buffer.trim().is_empty()
                && let Some(alias) = state.aliases.get_mut(index)
            {
                let new_name = new_buffer.trim().to_string();
                if alias.name != new_name {
                    alias.name = new_name;
                    sync_aliases_to_plugin(&state);
                    sync_current_alias(&state);
                }
            }
            state.rename_index = None;
            state.rename_buffer.clear();
        } else {
            state.rename_buffer = new_buffer;
        }
    }

    fn render_delete_confirm(&self, ctx: &egui::Context) {
        let delete_index = {
            let state = self.state.lock().unwrap();
            state.confirm_delete_index
        };
        let Some(index) = delete_index else {
            return;
        };

        let mut open = true;
        let mut confirm = false;
        let mut cancel = false;
        egui::Window::new("削除確認")
            .collapsible(false)
            .resizable(false)
            .open(&mut open)
            .show(ctx, |ui| {
                let name = {
                    let state = self.state.lock().unwrap();
                    state
                        .aliases
                        .get(index)
                        .map(|a| a.name.clone())
                        .unwrap_or_else(|| "(unknown)".to_string())
                };
                ui.label(format!("エイリアス \"{name}\" を削除しますか？"));
                ui.horizontal(|ui| {
                    if ui.button("削除").clicked() {
                        confirm = true;
                    }
                    if ui.button("キャンセル").clicked() {
                        cancel = true;
                    }
                });
            });

        if confirm || cancel {
            open = false;
        }

        let mut state = self.state.lock().unwrap();
        if !open {
            if confirm && index < state.aliases.len() {
                state.aliases.remove(index);
                if let Some(selected) = state.selected_index {
                    if selected == index {
                        state.selected_index = None;
                    } else if selected > index {
                        state.selected_index = Some(selected - 1);
                    }
                }
                sync_aliases_to_plugin(&state);
                sync_current_alias(&state);
            }
            state.confirm_delete_index = None;
        }
    }

    fn ensure_child_window_style(&self, child_hwnd: HWND) {
        unsafe {
            let style = GetWindowLongPtrW(child_hwnd, GWL_STYLE) as u32;
            let new_style = (style
                & !WS_POPUP.0
                & !WS_BORDER.0
                & !WS_DLGFRAME.0
                & !WS_CAPTION.0
                & !WS_THICKFRAME.0
                & !WS_MINIMIZEBOX.0
                & !WS_MAXIMIZEBOX.0
                & !WS_SYSMENU.0)
                | WS_CHILD.0
                | WS_VISIBLE.0;
            if style != new_style {
                SetWindowLongPtrW(child_hwnd, GWL_STYLE, new_style as isize);
            }

            let ex_style = GetWindowLongPtrW(child_hwnd, GWL_EXSTYLE) as u32;
            let new_ex_style = ex_style
                & !WS_EX_CLIENTEDGE.0
                & !WS_EX_WINDOWEDGE.0
                & !WS_EX_DLGMODALFRAME.0
                & !WS_EX_STATICEDGE.0;
            if ex_style != new_ex_style {
                SetWindowLongPtrW(child_hwnd, GWL_EXSTYLE, new_ex_style as isize);
            }

            if style != new_style || ex_style != new_ex_style {
                let _ = SetWindowPos(
                    child_hwnd,
                    None,
                    0,
                    0,
                    0,
                    0,
                    SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_FRAMECHANGED,
                );
            }
        }
    }

    fn embed_window_if_needed(&mut self, frame: &eframe::Frame) {
        if self.parent_hwnd == 0 {
            return;
        }
        if let Some(child_hwnd) = self.child_hwnd {
            self.ensure_child_window_style(HWND(child_hwnd as *mut c_void));
            self.resize_child_window(child_hwnd);
            return;
        }
        let Ok(handle) = frame.window_handle() else {
            log::warn!("Failed to get window handle for embedding.");
            return;
        };
        let RawWindowHandle::Win32(handle) = handle.as_raw() else {
            log::warn!("Unsupported window handle type for embedding.");
            return;
        };
        let child_hwnd = HWND(handle.hwnd.get() as *mut c_void);
        let parent_hwnd = HWND(self.parent_hwnd as *mut c_void);
        unsafe {
            let _ = SetParent(child_hwnd, parent_hwnd);
            self.ensure_child_window_style(child_hwnd);
            let _ = ShowWindow(child_hwnd, SW_SHOW);
        }
        self.embedded = true;
        self.child_hwnd = Some(child_hwnd.0 as isize);
        self.resize_child_window(child_hwnd.0 as isize);
    }

    fn resize_child_window(&self, child_hwnd: isize) {
        let parent_hwnd = HWND(self.parent_hwnd as *mut c_void);
        let child_hwnd = HWND(child_hwnd as *mut c_void);
        unsafe {
            let mut rect = RECT::default();
            if GetClientRect(parent_hwnd, &mut rect).is_ok() {
                let width = rect.right - rect.left;
                let height = rect.bottom - rect.top;
                let _ = MoveWindow(child_hwnd, 0, 0, width, height, true);
            }
        }
    }
}

impl eframe::App for LocalAliasUiApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.set_repaint_context(ctx);
        self.embed_window_if_needed(frame);

        let mut add_clicked = false;
        let mut info_clicked = false;
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Rusty Local Alias Plugin");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("？").clicked() {
                        info_clicked = true;
                    }
                    if ui.button("＋").clicked() {
                        add_clicked = true;
                    }
                });
            });
        });

        if info_clicked && let Ok(mut state) = self.state.lock() {
            state.show_info = true;
        }
        if add_clicked {
            self.handle_add_alias();
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            self.render_alias_list(ui);
        });

        self.render_info_modal(ctx);
        self.render_rename_modal(ctx);
        self.render_delete_confirm(ctx);
    }
}

enum UiAction {
    Select(usize),
    StartRename(usize),
    ConfirmDelete(usize),
    Move(usize, i32),
}

fn sync_aliases_to_plugin(state: &UiState) {
    LocalAliasPlugin::with_instance_mut(|instance| {
        instance.aliases = state.aliases.clone();
    });
}

pub(crate) fn sync_current_alias(state: &UiState) {
    let mut current = CURRENT_ALIAS.lock().unwrap();
    *current = state
        .selected_index
        .and_then(|index| state.aliases.get(index).cloned());
}
