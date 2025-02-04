//! Demonstrate tilemap with an [Anchor] component.
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use rand::seq::SliceRandom;
use rand::thread_rng;

mod helpers;

fn color(s: &str) -> Color {
    Srgba::hex(s).expect("hex color").into()
}

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    #[cfg(all(not(feature = "atlas"), feature = "render"))] array_texture_loader: Res<
        ArrayTextureLoader,
    >,
) {
    commands.spawn(Camera2d);

    let texture_handle: Handle<Image> = asset_server.load("tiles.png");

    let map_size = TilemapSize { x: 12, y: 12 };

    // Create a tilemap entity a little early.
    // We want this entity early because we need to tell each tile which tilemap entity
    // it is associated with. This is done with the TilemapId component on each tile.
    // Eventually, we will insert the `TilemapBundle` bundle on the entity, which
    // will contain various necessary components, such as `TileStorage`.
    let tilemap_entity = commands.spawn_empty().id();

    // To begin creating the map we will need a `TileStorage` component.
    // This component is a grid of tile entities and is used to help keep track of individual
    // tiles in the world. If you have multiple layers of tiles you would have a tilemap entity
    // per layer, each with their own `TileStorage` component.
    let mut tile_storage = TileStorage::empty(map_size);

    let mut rng = thread_rng();
    let colors: Vec<Color> = vec![
        color("FFBE0B"),
        color("FB5607"),
        color("FF006E"),
        color("8338EC"),
        color("3A86FF"),
    ];

    // Spawn the elements of the tilemap.
    // Alternatively, you can use helpers::filling::fill_tilemap.
    for x in 0..map_size.x {
        for y in 0..map_size.y {
            let tile_pos = TilePos { x, y };
            let tile_entity = commands
                .spawn(TileBundle {
                    position: tile_pos,
                    tilemap_id: TilemapId(tilemap_entity),
                    color: TileColor(*colors.choose(&mut rng).unwrap()),
                    ..Default::default()
                })
                .id();
            tile_storage.set(&tile_pos, tile_entity);
        }
    }

    let tile_size = TilemapTileSize { x: 16.0, y: 16.0 };
    let grid_size = tile_size.into();
    let map_type = TilemapType::default();

    // The tilemap is placed at the origin, but its anchor can change.
    commands.entity(tilemap_entity).insert(
        TilemapBundle {
            grid_size,
            map_type,
            size: map_size,
            storage: tile_storage,
            texture: TilemapTexture::Single(texture_handle),
            tile_size,
            transform: Transform::IDENTITY,
            anchor: TilemapAnchor::TopLeft,
            ..Default::default()
        });

    // Add atlas to array texture loader so it's preprocessed before we need to use it.
    // Only used when the atlas feature is off and we are using array textures.
    #[cfg(all(not(feature = "atlas"), feature = "render"))]
    {
        array_texture_loader.add(TilemapArrayTexture {
            texture: TilemapTexture::Single(asset_server.load("tiles.png")),
            tile_size,
            ..Default::default()
        });
    }
}

fn mark_origin(mut gizmos: Gizmos) {
    gizmos.axes_2d(Transform::IDENTITY, 1000.0);
}

fn change_anchor(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut TilemapAnchor, With<TilemapTexture>>,
    text: Single<Entity, With<Text>>,
    mut writer: TextUiWriter,
) {
    use TilemapAnchor::*;
    if keyboard_input.just_pressed(KeyCode::Space) {
        for mut anchor in &mut query {
            *anchor = match *anchor {
                TopLeft => TopCenter,
                TopCenter => TopRight,
                TopRight => CenterRight,
                CenterRight => BottomRight,
                BottomRight => BottomCenter,
                BottomCenter => BottomLeft,
                BottomLeft => CenterLeft,
                CenterLeft => Center,
                Center => Custom(Vec2::splat(0.25)),
                Custom(_) => None,
                None => TopLeft,
                };
            *writer.text(*text, 1) = format!("{:?}", *anchor);
        }
    }
}

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: String::from("Anchor Example - Press Space to change anchor."),
                        resolution: Vec2::splat(450.0).into(),
                        ..Default::default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins(TilemapPlugin)
        .add_systems(Startup, startup)
        .add_systems(Startup, setup_header)
        .add_systems(Update, helpers::camera::movement)
        .add_systems(Update, change_anchor)
        .add_systems(Update, mark_origin)
        .run();
}

fn setup_header(mut commands: Commands) {
    let font_size = 15.0;
    commands
        .spawn((
            Text::new("Anchor: "),
            TextLayout::new_with_justify(JustifyText::Center),
            TextFont {
                font_size,
                ..default()
            },
            Node {
                top: Val::Px(10.0),
                left: Val::Px(10.0),
                ..default()
            },
        ))
        .with_child((
            TextSpan::new("TopLeft"),
            TextFont {
                font_size,
                ..default()
            },
        ));
}
