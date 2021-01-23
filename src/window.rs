use bevy::prelude::*;
use bevy::window::WindowResized;

#[derive(Default)]
pub struct WindowPlugin;

impl Plugin for WindowPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_resource(WindowDescriptor {
            title: "Atomic Bomberman".to_string(),
            width: 640. * 2.,
            height: 480. * 2.,
            ..Default::default()
        })
        .add_resource(ClearColor(Color::rgb(0.04, 0.04, 0.04)))
        .add_startup_system(setup.system())
        .add_system(window_resize.system());
    }
}

pub struct WindowTransform;

fn setup(commands: &mut Commands) {
    commands
        .spawn(Camera2dBundle::default())
        .with(WindowTransform);
}

fn window_resize(
    resize_event: Res<Events<WindowResized>>,
    mut query: Query<&mut Transform, With<WindowTransform>>,
) {
    let mut event_reader = resize_event.get_reader();
    for event in event_reader.iter(&resize_event) {
        for mut transform in query.iter_mut() {
            transform.scale.x = 640. / event.width;
            transform.scale.y = 480. / event.height;
            transform.translation.x = 320.;
            transform.translation.y = -240.;
            // info!("window resize: {:?} {:?}", event, transform);
        }
    }
}
