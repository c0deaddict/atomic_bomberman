use bevy::prelude::*;
use bevy::window::WindowResized;

#[derive(Default)]
pub struct WindowPlugin;

impl Plugin for WindowPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.insert_resource(WindowDescriptor {
            title: "Atomic Bomberman".to_string(),
            width: 640. * 2.,
            height: 480. * 2.,
            ..Default::default()
        })
        .insert_resource(ClearColor(Color::rgb(0.04, 0.04, 0.04)))
        .add_startup_system(setup.system())
        .add_system(window_resize.system());
    }
}

pub struct WindowTransform;

fn setup(mut commands: Commands) {
    commands
        .spawn_bundle(OrthographicCameraBundle::new_2d())
        .insert(WindowTransform);
}

fn window_resize(
    mut resize_events_reader: EventReader<WindowResized>,
    mut query: Query<&mut Transform, With<WindowTransform>>,
) {
    for event in resize_events_reader.iter() {
        for mut transform in query.iter_mut() {
            transform.scale.x = 640. / event.width;
            transform.scale.y = 480. / event.height;
            transform.translation.x = 320.;
            transform.translation.y = -240.;
            // info!("window resize: {:?} {:?}", event, transform);
        }
    }
}
