// SPDX-FileCopyrightText: 2022 Harish Rajagopal <harish.rajagopals@gmail.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Templates for various GUI components

use gtk::prelude::*;
use relm4::{gtk, RelmWidgetExt, WidgetTemplate};

/// Button that ends the greeter (eg. Reboot)
#[relm4::widget_template(pub)]
impl WidgetTemplate for EndButton {
    view! {
        gtk::Button {
            set_focusable: true,
            add_css_class: "destructive-action",
        }
    }
}

/// Label for an entry/combo box
#[relm4::widget_template(pub)]
impl WidgetTemplate for EntryLabel {
    view! {
        gtk::Label {
            set_width_request: 100,
            set_xalign: 1.0,
        }
    }
}

/// Main UI of the greeter
#[relm4::widget_template(pub)]
impl WidgetTemplate for Ui {
    view! {
        gtk::Overlay {
            /// Background image
            #[name = "background"]
            gtk::Picture,

            /// Main login box
            add_overlay = &gtk::Frame {
                set_halign: gtk::Align::Center,
                set_valign: gtk::Align::End,
                set_margin_bottom: 100,
                inline_css: "background-color: transparent; border: none;",

                gtk::Grid {
                    set_column_spacing: 15,
                    set_margin_bottom: 15,
                    set_margin_end: 15,
                    set_margin_start: 15,
                    set_margin_top: 15,
                    set_row_spacing: 15,

                    /// Widget where the user enters a secret
                    #[name = "secret_entry"]
                    attach[0, 0, 1, 1] = &gtk::PasswordEntry {
                        set_show_peek_icon: true,
                        set_width_request: 125,
                        set_height_request: 32,
                        set_halign: gtk::Align::Center,
                        inline_css: "
                        outline: 3px solid rgba(35, 51, 113, 1);
                        border-radius: 16px;
                        background-color: rgba(229, 215, 230, 1);
                        padding: 0px 16px 0px 16px;
                        font-size: 16px;
                        ",
                     },
                },
            },

            /// Clock widget
            add_overlay = &gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_halign: gtk::Align::Center,
                set_valign: gtk::Align::Start,
                set_margin_top: 80,
                // Make it fit cleanly onto the top edge of the screen.
                inline_css: "
                    border: none;
                    color: rgba(35, 51, 113, 1);
                    background-color: transparent;
                ",

                /// Label displaying the current date & time
                #[name = "date_label"]
                gtk::Label {
                    set_use_markup: true,
                    inline_css: "
                    font-size: 28px;
                    font-family: JetbrainsMono Nerd Font;
                    "
                 },
                /// Label displaying the current date & time
                #[name = "time_label"]
                gtk::Label {
                    set_use_markup: true,
                    inline_css: "
                    font-size: 77px;
                    font-family: JetbrainsMono Nerd Font;
                    "
                 },
            },

            /// Collection of widgets appearing at the bottom
            add_overlay = &gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_halign: gtk::Align::Center,
                set_valign: gtk::Align::End,
                set_margin_bottom: 15,
                set_spacing: 15,


                /// Notification bar for error messages
                #[name = "error_info"]
                gtk::InfoBar {
                    // During init, the info bar closing animation is shown. To hide that, make
                    // it invisible. Later, the code will permanently make it visible, so that
                    // `InfoBar::set_revealed` will work properly with animations.
                    set_visible: false,
                    set_margin_bottom: 200,
                    set_message_type: gtk::MessageType::Error,

                    /// The actual error message
                    #[name = "error_label"]
                    gtk::Label {
                        set_halign: gtk::Align::Center,
                        set_margin_top: 10,
                        set_margin_bottom: 10,
                        set_margin_start: 10,
                        set_margin_end: 10,
                        inline_css: "
                        ",
                    },
                },

                /// Collection of buttons that close the greeter (eg. Reboot)
                gtk::Box {
                    set_halign: gtk::Align::Center,
                    set_homogeneous: true,
                    set_spacing: 15,

                    /// Button to reboot
                    #[name = "reboot_button"]
                    #[template]
                    EndButton {
                         set_label: "Reboot",
                         inline_css: "
                         border: none;
                         border-radius: 20px;
                         font-size: 16px;
                         ",
                    },

                    /// Button to power-off
                    #[name = "poweroff_button"]
                    #[template]
                    EndButton {
                        set_label: "Power Off",
                        inline_css: "
                        border: none;
                        border-radius: 20px;
                        font-size: 16px;
                        ",
                 },
                },
            },
        }
    }
}
