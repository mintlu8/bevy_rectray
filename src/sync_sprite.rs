use bevy::{
    app::{Plugin, PostUpdate},
    asset::Assets,
    ecs::{
        change_detection::DetectChanges,
        schedule::IntoSystemConfigs,
        system::{Query, Res},
    },
    image::Image,
    math::Vec2,
    sprite::{Sprite, TextureAtlas, TextureAtlasLayout},
};

use crate::{compute_transform_2d, transform::SyncDimension, Dimension, RectrayTransformSet};

fn get_atlas_size(
    handle: &Option<TextureAtlas>,
    layouts: &Assets<TextureAtlasLayout>,
) -> Option<Vec2> {
    handle
        .as_ref()
        .map(|atlas| match layouts.get(atlas.layout.id()) {
            Some(layout) => layout
                .textures
                .get(atlas.index)
                .map(|x| x.size().as_vec2())
                .unwrap_or(Vec2::ZERO),
            None => Vec2::ZERO,
        })
}

/// System that synchronizes dimension between [`Sprite`] and [`Dimension`].
///
/// # Limitations
///
/// For [`SyncDimension::ToDimension`],
/// the user must manually trigger change detection on [`Sprite`] after the following actions:
///
/// * Modifying the referenced [`Image`] or [`TextureAtlasLayout`].
pub fn sync_sprite_dimension(
    images: Res<Assets<Image>>,
    atlases: Res<Assets<TextureAtlasLayout>>,
    mut query: Query<(&mut Sprite, &mut Dimension, &SyncDimension)>,
) {
    for (mut sprite, mut dimension, sync_mode) in &mut query {
        if !dimension.is_changed() && !sprite.is_changed() {
            continue;
        }
        match sync_mode {
            SyncDimension::None => continue,
            SyncDimension::ToDimension => {
                dimension.0 = sprite
                    .custom_size
                    .or_else(|| get_atlas_size(&sprite.texture_atlas, &atlases))
                    .or_else(|| images.get(sprite.image.id()).map(|x| x.size_f32()))
                    .unwrap_or(Vec2::ZERO);
            }
            SyncDimension::FromDimension => {
                sprite.custom_size = Some(dimension.0);
            }
            _ => {
                let mut image_size = get_atlas_size(&sprite.texture_atlas, &atlases)
                    .or_else(|| images.get(sprite.image.id()).map(|x| x.size_f32()))
                    .unwrap_or(Vec2::ONE);
                if image_size.x <= 0.0 || image_size.y <= 0.0 {
                    image_size = Vec2::ONE;
                }
                let fac = match sync_mode {
                    SyncDimension::FromAspectDimensionX => image_size.x,
                    SyncDimension::FromAspectDimensionY => image_size.y,
                    _ => image_size.x.max(image_size.y),
                };
                let ratio = image_size / fac;
                sprite.custom_size = Some(dimension.0 * ratio);
            }
        }
    }
}

pub struct SyncSpritePlugin;

impl Plugin for SyncSpritePlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(
            PostUpdate,
            sync_sprite_dimension
                .in_set(RectrayTransformSet)
                .before(compute_transform_2d),
        );
    }
}
