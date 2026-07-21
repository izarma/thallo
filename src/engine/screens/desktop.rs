use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, EguiTextureHandle, egui};

use crate::engine::{UiPassSystems, asset_tracking::LoadResource, system_apps::Applications};

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<IconAssets>()
        .load_resource::<DesktopAssets>()
        .load_resource::<WindowAssets>()
        .add_systems(
            EguiPrimaryContextPass,
            (
                cache_icon_textures.run_if(
                    resource_exists::<IconAssets>.and(not(resource_exists::<IconTextures>)),
                ),
                cache_desktop_textures.run_if(
                    resource_exists::<DesktopAssets>
                        .and(resource_exists::<WindowAssets>)
                        .and(not(resource_exists::<DesktopTextures>)),
                ),
            )
                .chain()
                .in_set(UiPassSystems::CacheTextures),
        );
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
struct IconAssets {
    #[dependency]
    folder: Handle<Image>,
    #[dependency]
    txt: Handle<Image>,
    #[dependency]
    png: Handle<Image>,
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct DesktopAssets {
    #[dependency]
    pub wallpaper: Handle<Image>,
    #[dependency]
    pub main_cursor: Handle<Image>,
    #[dependency]
    pub load_cursor: Handle<Image>,
    #[dependency]
    pub folder_minimized: Handle<Image>,
    #[dependency]
    pub txt_minimized: Handle<Image>,
    #[dependency]
    pub png_minimized: Handle<Image>,
    #[dependency]
    pub terminal_minimized: Handle<Image>,
    #[dependency]
    pub chat_minimized: Handle<Image>,
    #[dependency]
    pub hack_minimized: Handle<Image>,
    #[dependency]
    pub start_btn: Handle<Image>,
    #[dependency]
    pub msg_notification: Handle<AudioSource>,
    #[dependency]
    pub file_transfer_sheet: Handle<Image>,
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct WindowAssets {
    #[dependency]
    pub window: Handle<Image>,
    #[dependency]
    pub chatbox: Handle<Image>,
    #[dependency]
    pub terminal: Handle<Image>,
}

impl FromWorld for IconAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            folder: assets.load("ui/icons/folder.png"),
            txt: assets.load("ui/icons/txt.png"),
            png: assets.load("ui/icons/png.png"),
        }
    }
}

impl FromWorld for DesktopAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            wallpaper: assets.load("ui/desktop.png"),
            main_cursor: assets.load("ui/cursors/cursor.png"),
            load_cursor: assets.load("ui/cursors/cursor_loading.png"),
            folder_minimized: assets.load("ui/tabs/folder_minimized.png"),
            txt_minimized: assets.load("ui/tabs/txt_minimized.png"),
            png_minimized: assets.load("ui/tabs/png_minimized.png"),
            terminal_minimized: assets.load("ui/tabs/terminal_minimized.png"),
            chat_minimized: assets.load("ui/tabs/chatbox_minimized.png"),
            hack_minimized: assets.load("ui/tabs/hack_minimized.png"),
            start_btn: assets.load("ui/start_button.png"),
            msg_notification: assets.load("audio/sfx/notification.ogg"),
            file_transfer_sheet: assets.load("ui/file_transmition_sheet.png"),
        }
    }
}

impl FromWorld for WindowAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            window: assets.load("ui/window.png"),
            chatbox: assets.load("ui/chatbox.png"),
            terminal: assets.load("ui/terminal.png"),
        }
    }
}

#[derive(Resource, Clone, Copy)]
pub struct IconTextures {
    pub folder: egui::TextureId,
    pub txt: egui::TextureId,
    pub png: egui::TextureId,
}

#[derive(Resource, Clone, Copy)]
pub struct DesktopTextures {
    pub wallpaper: egui::TextureId,
    pub start: egui::TextureId,
    pub folder_minimized: egui::TextureId,
    pub txt_minimized: egui::TextureId,
    pub png_minimized: egui::TextureId,
    pub terminal_minimized: egui::TextureId,
    pub chat_minimized: egui::TextureId,
    pub hack_minimized: egui::TextureId,
    pub window: egui::TextureId,
    pub terminal: egui::TextureId,
    pub chatbox: egui::TextureId,
    pub file_transfer_sheet: egui::TextureId,
}

impl DesktopTextures {
    /// Pick the correct minimised-tab icon for an open window's [`Applications`] kind.
    pub fn icon_for_app(&self, kind: &Applications) -> egui::TextureId {
        match kind {
            Applications::FileExplorer { .. } => self.folder_minimized,
            Applications::ImageViewer { .. } => self.png_minimized,
            Applications::TextViewer { .. } => self.txt_minimized,
            Applications::Terminal { .. } => self.terminal_minimized,
            Applications::Chatbox { .. } => self.chat_minimized,
            Applications::Decrypter { .. }
            | Applications::Ripper { .. }
            | Applications::Unlocker { .. } => self.hack_minimized,
        }
    }
}

fn cache_icon_textures(mut contexts: EguiContexts, icons: Res<IconAssets>, mut cmd: Commands) {
    let textures = IconTextures {
        folder: contexts.add_image(EguiTextureHandle::Weak(icons.folder.id())),
        txt: contexts.add_image(EguiTextureHandle::Weak(icons.txt.id())),
        png: contexts.add_image(EguiTextureHandle::Weak(icons.png.id())),
    };
    cmd.insert_resource(textures);
}

fn cache_desktop_textures(
    mut contexts: EguiContexts,
    d_ass: Res<DesktopAssets>,
    w_ass: Res<WindowAssets>,
    mut cmd: Commands,
) {
    let textures = DesktopTextures {
        wallpaper: contexts.add_image(EguiTextureHandle::Weak(d_ass.wallpaper.id())),
        start: contexts.add_image(EguiTextureHandle::Weak(d_ass.start_btn.id())),
        folder_minimized: contexts.add_image(EguiTextureHandle::Weak(d_ass.folder_minimized.id())),
        txt_minimized: contexts.add_image(EguiTextureHandle::Weak(d_ass.txt_minimized.id())),
        png_minimized: contexts.add_image(EguiTextureHandle::Weak(d_ass.png_minimized.id())),
        terminal_minimized: contexts
            .add_image(EguiTextureHandle::Weak(d_ass.terminal_minimized.id())),
        chat_minimized: contexts.add_image(EguiTextureHandle::Weak(d_ass.chat_minimized.id())),
        hack_minimized: contexts.add_image(EguiTextureHandle::Weak(d_ass.hack_minimized.id())),
        window: contexts.add_image(EguiTextureHandle::Weak(w_ass.window.id())),
        terminal: contexts.add_image(EguiTextureHandle::Weak(w_ass.terminal.id())),
        chatbox: contexts.add_image(EguiTextureHandle::Weak(w_ass.chatbox.id())),
        file_transfer_sheet: contexts
            .add_image(EguiTextureHandle::Weak(d_ass.file_transfer_sheet.id())),
    };
    cmd.insert_resource(textures);
    info!("Desktop cached");
}
