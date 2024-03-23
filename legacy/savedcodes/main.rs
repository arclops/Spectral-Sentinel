use druid::widget::{Button, Flex, CrossAxisAlignment, Label, TextBox, Align};
use druid::{AppLauncher, Data, Lens, LocalizedString, Widget, WindowDesc, WidgetExt};
use std::thread;

const VERTICAL_WIDGET_SPACING: f64 = 15.0;
const TEXT_BOX_WIDTH: f64 = 250.0;
const WINDOW_TITLE: LocalizedString<AppState> = LocalizedString::new("Password Protection");

#[derive(Clone, Data, Lens)]
struct AppState {
    password: String,
}

fn main() {
    let main_window = WindowDesc::new(build_root_widget())
        .title(WINDOW_TITLE)
        .window_size((400.0, 400.0));

    let initial_state = AppState {
        password: String::new(),
    };

    AppLauncher::with_window(main_window)
        .launch(initial_state)
        .expect("Failed to launch application");
}

fn build_root_widget() -> impl Widget<AppState> {
    let instruction_label = Label::new("Enter the password to terminate:");
    let password_textbox = TextBox::new()
        .with_placeholder("Enter the password")
        .fix_width(TEXT_BOX_WIDTH)
        .lens(AppState::password);

    let ok_button = Button::new("Ok").on_click(move |_ctx, data: &mut AppState, _env| {
        // Spawn a new thread for password checking
        let data_password = data.password.clone();
        thread::spawn(move || {
            if data_password == "Admin@123" {
                println!("Password matched. Exiting.");
                if let Err(err) = sender.send(()) {
                    eprintln!("Failed to send termination signal: {}", err);
                }
                break;
            } else {
                // Replace the println! statement with the error dialog function
                show_error_dialog("Incorrect password. Please try again.");
            }
        });
    });

    let cancel_button = Button::new("Cancel").on_click(|_ctx, _data: &mut AppState, _env| {
        println!("Cancelled. Exiting.");
        std::process::exit(0);
    });

    // Group widgets and center both horizontally and vertically using Align
    let layout = Align::centered(
        Flex::column()
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .with_child(instruction_label)
            .with_spacer(VERTICAL_WIDGET_SPACING)
            .with_child(password_textbox)
            .with_spacer(VERTICAL_WIDGET_SPACING)
            .with_child(
                Flex::row()
                    .cross_axis_alignment(CrossAxisAlignment::Center)
                    .with_child(ok_button)
                    .with_spacer(20.0)
                    .with_child(cancel_button),
            ),
    );

    layout
}

// Function to show an error dialog using druid
fn show_error_dialog(message: &str) {
    // In a real application, you would use druid's dialog API to create and show an error dialog.
    // For simplicity, this example just prints the error message.
    println!("Error: {}", message);
}
