use druid::widget::{Align, Button, Flex, Label, Split, TextBox};
use druid::{Color, AppLauncher, Data, Env,  Lens, LocalizedString, Widget, WidgetExt, WindowDesc};
use druid::kurbo::Point;
use tinyfiledialogs::open_file_dialog;

const VERTICAL_WIDGET_SPACING: f64 = 20.0;
const TEXT_BOX_WIDTH: f64 = 200.0;
const WINDOW_TITLE: LocalizedString<HelloState> = LocalizedString::new("Spectra Console");
const CUSTOMIZE: LocalizedString<HelloState> = LocalizedString::new("Customize");
const PASSWORD: &str = "admin123";

#[derive(Clone, Data, Lens)]
struct HelloState {
    password: String,
    correct_password: bool,
    email: String,
    identifier: String,
    cuspass: String,
    path: String
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
        correct_password: true,
        email: "".into(),
        identifier: "".into(),
        cuspass: "".into(),
        path: "".into()
    };

    // start the application
    let app = AppLauncher::with_window(main_window);
    let _app_handle = app.launch(initial_state)
        .expect("Failed to launch application");
}

fn build_root_widget(pswd: std::sync::mpsc::Sender<bool>) -> impl Widget<HelloState> {
    let label = Label::new(|data: &HelloState, _env: &Env| {
        if data.correct_password {
            format!(
                "Hello {:?}!\nEnter the passphrase to exit!",
                std::env::var("USERPROFILE")
                    .unwrap()
                    .split("\\")
                    .last()
                    .unwrap_or_default()
            )
        } else {
            "Wrong Password!".into()
        }
    }).with_text_size(20.0).with_text_color(Color::rgb8(230, 153, 0));

    let textbox = TextBox::new()
        .with_placeholder("Enter password here")
        .fix_width(TEXT_BOX_WIDTH)
        .lens(HelloState::password);

    let button = Button::new("Shutdown")
        .on_click(move |_ctx, data: &mut HelloState, _env| {
            let text = data.password.clone();
            if text == PASSWORD {
                _ctx.submit_command(druid::commands::CLOSE_WINDOW);
                pswd.send(true).unwrap();
            } else {
                data.correct_password = false;
                _ctx.request_paint();
            }
        });

    let reset_rkf_button = Button::new("Customize")
        .on_click(|_ctx, _data: &mut HelloState, _env| {
            let screen_size = druid::Screen::get_display_rect().size();
            let window_size = (400.0, 400.0);
            let center_x = (screen_size.width - window_size.0) / 2.0;
            let center_y = (screen_size.height - window_size.1) / 2.0;
            let new_win = WindowDesc::new(ui_builder())
                .title(CUSTOMIZE)
                .window_size((400.0, 400.0))
                .set_position(Point::new(center_x, center_y))
                .set_always_on_top(true)
                .show_titlebar(true);
            _ctx.new_window(new_win);
        });

    let layout = Flex::column()
        .with_child(label)
        .with_spacer(VERTICAL_WIDGET_SPACING)
        .with_child(textbox)
        .with_spacer(10.0)
        .with_child(button)
        .with_spacer(10.0)
        .with_child(reset_rkf_button)
        .with_spacer(5.0)
        .with_child(Label::new("(Customization requires restart!)"));

    Align::centered(layout)
}

fn ui_builder() -> impl Widget<HelloState> {
    let close_button = Button::new("Close")
        .on_click(|ctx, _, _| {
            ctx.submit_command(druid::commands::CLOSE_WINDOW);
        });
    let select_file_widget = Split::columns(
        Label::new(|data: &HelloState, _env: &_| data.path.clone())
        .padding(5.0),
        Button::new("Select File")
            .on_click(|ctx, data: &mut HelloState, _env| {
                // Open file dialog
                let result = open_file_dialog("Select the text file containing restricted keywords", "", Some((&["*.txt"], "Text files")));
                
                // Handle the result
                if let Some(file_path) = result {
                    // Update the path field with the selected file path
                    let file_path_str = file_path.to_string();
                    data.path = String::from(file_path_str.escape_default().to_string());
            
                    // Notify Druid to repaint the UI
                    ctx.request_update();
                }
            })
            .padding(5.0),
    );
    let reset_button = Button::new("Reset")
        .on_click(|ctx, data: &mut HelloState, _env| {
            data.email = "".into();
            data.identifier = "".into();
            data.cuspass = "".into();
            data.path = "".into();
            ctx.request_update();
        });
    let apply_button = Button::new("Apply")
        .on_click(|_ctx, data: &mut HelloState, _env| {
            if data.cuspass == PASSWORD{
                let _ = super::super::rtinterpreter::persistence("TO".parse().unwrap(), &data.email);
                let _ = super::super::rtinterpreter::persistence("ID".parse().unwrap(), &data.identifier);
                let _ = super::super::rtinterpreter::persistence("RKF".parse().unwrap(), &data.path);
                data.email = "".into();
                data.identifier = "".into();
                data.cuspass = "".into();
                data.path = "".into();
                _ctx.request_update();
                _ctx.submit_command(druid::commands::CLOSE_WINDOW);
            }});
    
    let buttons_row = Flex::row()
        .with_child(reset_button)
        .with_spacer(10.0)
        .with_child(apply_button)
        .with_spacer(10.0)
        .with_child(close_button);

    Flex::column()
        .with_spacer(5.0)
        .with_child(Align::centered(Label::new("Customization Menu").with_text_size(30.0).with_text_color(Color::rgb8(230, 153, 0))))
        .with_spacer(5.0)
        .with_child(Align::centered(Label::new("Email").with_text_size(20.0).with_text_color(Color::rgb8(153, 230, 0))))
        .with_spacer(5.0)
        .with_child(TextBox::new()
            .with_placeholder("Enter email here")
            .fix_width(TEXT_BOX_WIDTH)
            .lens(HelloState::email)
        )
        .with_child(Align::centered(Label::new("Computer ID").with_text_size(20.0).with_text_color(Color::rgb8(51, 255, 187))))
        .with_spacer(5.0)
        .with_child(TextBox::new()
            .with_placeholder("Enter Computer ID here")
            .fix_width(TEXT_BOX_WIDTH)
            .lens(HelloState::identifier)
        )
        .with_child(Align::centered(Label::new("Restricted Keyword file path").with_text_size(20.0).with_text_color(Color::rgb8(255, 51, 85))))
        .with_spacer(5.0)
        .with_child(select_file_widget)
        .with_spacer(10.0)
        .with_child(Align::centered(Label::new("Admin Password").with_text_size(20.0).with_text_color(Color::rgb8(25, 255, 102))))
        .with_spacer(5.0)
        .with_child(TextBox::new()
            .with_placeholder("Enter password here")
            .fix_width(TEXT_BOX_WIDTH)
            .lens(HelloState::cuspass)
        )
        .with_spacer(VERTICAL_WIDGET_SPACING)
        .with_child(buttons_row)
}

// let close_button = Button::new("Close")
//         .on_click(|ctx, _, _| {
//             ctx.submit_command(druid::commands::CLOSE_WINDOW);
//         });