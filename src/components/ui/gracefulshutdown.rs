use druid::widget::{Align, Flex, Label, TextBox, Button};
use druid::{AppLauncher, Data, Env, Lens, LocalizedString, Widget, WidgetExt, WindowDesc};
use druid::kurbo::Point;

const VERTICAL_WIDGET_SPACING: f64 = 20.0;
const TEXT_BOX_WIDTH: f64 = 200.0;
const WINDOW_TITLE: LocalizedString<HelloState> = LocalizedString::new("Spectra Console");

#[derive(Clone, Data, Lens)]
struct HelloState {
    password: String,
    correct_password: bool,
}

pub fn gracefulshutdown(pswd: std::sync::mpsc::Sender<bool>) {
    let screen_size = druid::Screen::get_display_rect().size();
    let window_size = (400.0, 400.0);

    let center_x = (screen_size.width - window_size.0) / 2.0;
    let center_y = (screen_size.height - window_size.1) / 2.0;
    // describe the main window
    let main_window = WindowDesc::new(build_root_widget(pswd))
        .title(WINDOW_TITLE)
        .window_size((400.0, 400.0))
        .set_position(Point::new(center_x, center_y))
        .set_always_on_top(true)
        .show_titlebar(true);

    // create the initial app state
    let initial_state = HelloState {
        password: "".into(),
        correct_password: true
    };

    // start the application
    AppLauncher::with_window(main_window)
        .launch(initial_state)
        .expect("Failed to launch application");
}

fn build_root_widget(pswd: std::sync::mpsc::Sender<bool>) -> impl Widget<HelloState> {
    // a label that will determine its text based on the current app data.
    let label = Label::new(|data: &HelloState, _env: &Env| {
        if data.correct_password {
            format!(
                "Hello {:?}!\nEnter the passphrase to exit!",
                std::env::var("USERPROFILE").unwrap().split("\\").last().unwrap_or_default()
            )
        } else {
            "Wrong Password!".into()
        }
    });    // a textbox that modifies `password`.
    let textbox = TextBox::new()
        .with_placeholder("Enter password here")
        .fix_width(TEXT_BOX_WIDTH)
        .lens(HelloState::password);

    let button = Button::new("Shutdown")
    .on_click(move |_ctx, data: &mut HelloState, _env| {
        let text = data.password.clone(); // Clone the text from the state
        if text == "Hehe" {
            // Request the application to quit
            _ctx.submit_command(druid::commands::CLOSE_WINDOW);
            pswd.send(true).unwrap();
        }
        else {
            // Update state to indicate incorrect password
            data.correct_password = false;
            // Force widget to recompute
            _ctx.request_paint();
        }
    });

    // arrange the two widgets vertically, with some padding
    let layout = Flex::column()
        .with_child(label)
        .with_spacer(VERTICAL_WIDGET_SPACING)
        .with_child(textbox)
        .with_spacer(10.0)
        .with_child(button);

    // center the two widgets in the available space
    Align::centered(layout)
}
