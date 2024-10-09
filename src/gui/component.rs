// SPDX-FileCopyrightText: 2022 Harish Rajagopal <harish.rajagopals@gmail.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Setup for using the greeter as a Relm4 component

use std::path::PathBuf;
use std::time::Duration;

use chrono::Local;
use tracing::{debug, warn};

use gtk::prelude::*;
use relm4::{
    component::{AsyncComponent, AsyncComponentParts, AsyncComponentSender},
    gtk,
};
use tokio::time::sleep;

use super::messages::{CommandMsg, InputMsg};
use super::model::{Greeter, InputMode, Updates};
use super::templates::Ui;

const DATE_FMT: &str = "<b>%B %-d, %A</b>";
const TIME_FMT: &str = "<b><big>%R</big></b>";
const DATETIME_UPDATE_DELAY: u64 = 500;

/// Load GTK settings from the greeter config.
fn setup_settings(model: &Greeter, root: &gtk::ApplicationWindow) {
    let settings = root.settings();
    let config = if let Some(config) = model.config.get_gtk_settings() {
        config
    } else {
        return;
    };

    debug!(
        "Setting dark theme: {}",
        config.application_prefer_dark_theme
    );
    settings.set_gtk_application_prefer_dark_theme(config.application_prefer_dark_theme);

    if let Some(cursor_theme) = &config.cursor_theme_name {
        debug!("Setting cursor theme: {cursor_theme}");
        settings.set_gtk_cursor_theme_name(config.cursor_theme_name.as_deref());
    };

    if let Some(font) = &config.font_name {
        debug!("Setting font: {font}");
        settings.set_gtk_font_name(config.font_name.as_deref());
    };

    if let Some(icon_theme) = &config.icon_theme_name {
        debug!("Setting icon theme: {icon_theme}");
        settings.set_gtk_icon_theme_name(config.icon_theme_name.as_deref());
    };

    if let Some(theme) = &config.theme_name {
        debug!("Setting theme: {theme}");
        settings.set_gtk_theme_name(config.theme_name.as_deref());
    };
}

/// Set up auto updation for the datetime label.
fn setup_datetime_display(sender: &AsyncComponentSender<Greeter>) {
    // Set a timer in a separate thread that signals the main thread to update the time, so as to
    // not block the GUI.
    sender.command(|sender, shutdown| {
        shutdown
            .register(async move {
                // Run it infinitely, since the clock always needs to stay updated.
                loop {
                    if sender.send(CommandMsg::UpdateTime).is_err() {
                        warn!("Couldn't update datetime");
                    };
                    sleep(Duration::from_millis(DATETIME_UPDATE_DELAY)).await;
                }
            })
            .drop_on_shutdown()
    });
}

/// The info required to initialize the greeter
pub struct GreeterInit {
    pub config_path: PathBuf,
    pub css_path: PathBuf,
    pub demo: bool,
}

#[relm4::component(pub, async)]
impl AsyncComponent for Greeter {
    type Input = InputMsg;
    type Output = ();
    type Init = GreeterInit;
    type CommandOutput = CommandMsg;

    view! {
        // The `view!` macro needs a proper widget, not a template, as the root.
        #[name = "window"]
        gtk::ApplicationWindow {
            set_visible: true,

            // Name the UI widget, otherwise the inner children cannot be accessed by name.
            #[name = "ui"]
            #[template]
            Ui {
                #[template_child]
                background { set_filename: model.config.get_background().clone() },
                #[template_child]
                date_label {
                    #[track(model.updates.changed(Updates::date()))]
                    set_label: &model.updates.date
                },
                #[template_child]
                time_label {
                    #[track(model.updates.changed(Updates::time()))]
                    set_label: &model.updates.time
                },
                #[template_child]
                secret_entry {
                    grab_focus: (),
                    #[track(model.updates.changed(Updates::input()))]
                    set_text: &model.updates.input,
                    connect_activate[
                        sender
                    ] => move |this| {
                        sender.input(Self::Input::Login {
                            input: this.text().to_string(),
                        })
                    }
                },
                #[template_child]
                error_info {
                    #[track(model.updates.changed(Updates::error()))]
                    set_revealed: model.updates.error.is_some(),
                },
                #[template_child]
                error_label {
                    #[track(model.updates.changed(Updates::error()))]
                    set_label: model.updates.error.as_ref().unwrap_or(&"".to_string()),
                },
                #[template_child]
                reboot_button { connect_clicked => Self::Input::Reboot },
                #[template_child]
                poweroff_button { connect_clicked => Self::Input::PowerOff },
            }
        }
    }

    fn post_view() {
        if model.updates.changed(Updates::monitor()) {
            if let Some(monitor) = &model.updates.monitor {
                widgets.window.fullscreen_on_monitor(monitor);
                // For some reason, the GTK settings are reset when changing monitors, so re-apply them.
                setup_settings(self, &widgets.window);
            }
        }
    }

    /// Initialize the greeter.
    async fn init(
        input: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let mut model = Self::new(&input.config_path, input.demo).await;
        let widgets = view_output!();

        // Make the info bar permanently visible, since it was made invisible during init. The
        // actual visuals are controlled by `InfoBar::set_revealed`.
        widgets.ui.error_info.set_visible(true);

        // cfg directives don't work inside Relm4 view! macro.
        #[cfg(feature = "gtk4_8")]
        widgets
            .ui
            .background
            .set_content_fit(match model.config.get_background_fit() {
                BgFit::Fill => gtk4::ContentFit::Fill,
                BgFit::Contain => gtk4::ContentFit::Contain,
                BgFit::Cover => gtk4::ContentFit::Cover,
                BgFit::ScaleDown => gtk4::ContentFit::ScaleDown,
            });

        // Cancel any previous session, just in case someone started one.
        if let Err(err) = model.greetd_client.lock().await.cancel_session().await {
            warn!("Couldn't cancel greetd session: {err}");
        };

        model.choose_monitor(widgets.ui.display().name().as_str(), &sender);
        if let Some(monitor) = &model.updates.monitor {
            // The window needs to be manually fullscreened, since the monitor is `None` at widget
            // init.
            root.fullscreen_on_monitor(monitor);
        } else {
            // Couldn't choose a monitor, so let the compositor choose it for us.
            root.fullscreen();
        }

        // For some reason, the GTK settings are reset when changing monitors, so apply them after
        // full-screening.
        setup_settings(&model, &root);
        setup_datetime_display(&sender);

        if input.css_path.exists() {
            debug!("Loading custom CSS from file: {}", input.css_path.display());
            let provider = gtk::CssProvider::new();
            provider.load_from_path(input.css_path);
            gtk::StyleContext::add_provider_for_display(
                &widgets.ui.display(),
                &provider,
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        };

        AsyncComponentParts { model, widgets }
    }

    async fn update(
        &mut self,
        msg: Self::Input,
        sender: AsyncComponentSender<Self>,
        _root: &Self::Root,
    ) {
        debug!("Got input message: {msg:?}");

        // Reset the tracker for update changes.
        self.updates.reset();

        match msg {
            Self::Input::Login { input } => self.login_click_handler(&sender, input).await,
            Self::Input::Reboot => self.reboot_click_handler(&sender),
            Self::Input::PowerOff => self.poweroff_click_handler(&sender),
        }
    }

    /// Perform the requested changes when a background task sends a message.
    async fn update_cmd(
        &mut self,
        msg: Self::CommandOutput,
        sender: AsyncComponentSender<Self>,
        _root: &Self::Root,
    ) {
        if !matches!(msg, Self::CommandOutput::UpdateTime) {
            debug!("Got command message: {msg:?}");
        }

        // Reset the tracker for update changes.
        self.updates.reset();

        match msg {
            Self::CommandOutput::UpdateTime => {
                self.updates
                    .set_date(Local::now().format(DATE_FMT).to_string());
                self.updates
                    .set_time(Local::now().format(TIME_FMT).to_string());
            }
            Self::CommandOutput::ClearErr => self.updates.set_error(None),
            Self::CommandOutput::HandleGreetdResponse(response) => {
                self.handle_greetd_response(&sender, response).await
            }
            Self::CommandOutput::MonitorRemoved(display_name) => {
                self.choose_monitor(display_name.as_str(), &sender)
            }
        };
    }
}
