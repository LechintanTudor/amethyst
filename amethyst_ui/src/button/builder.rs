use crate::{
    Anchor, FontAsset, Parent, Stretch, UiButton, UiButtonAction, UiButtonActionType,
    UiButtonActionRetrigger, UiImage, UiText, UiTransform,
};
use amethyst_assets::Handle;
use amethyst_core::{
    ecs::{
        prelude::*,
        storage::Component,
    },
};
use amethyst_rendy::palette::Srgba;
use smallvec::SmallVec;

const DEFAULT_Z: f32 = 1.0;
const DEFAULT_WIDTH: f32 = 128.0;
const DEFAULT_HEIGHT: f32 = 64.0;
const DEFAULT_FONT_SIZE: f32 = 32.0;
const DEFAULT_TEXT_COLOR: (f32, f32, f32, f32) = (0.0, 0.0, 0.0, 1.0);
const DEFAULT_BACKGROUND_COLOR: (f32, f32, f32, f32) = (0.8, 0.8, 0.8, 1.0);

pub trait UiButtonBuilderTarget {
    fn create_entity(&mut self) -> Entity;

    fn add_component<C>(&mut self, entity: Entity, component: C)
    where
        C: Component;
}

impl UiButtonBuilderTarget for World {
    fn create_entity(&mut self) -> Entity {
        self.insert((), Some(()))[0]
    }

    fn add_component<C>(&mut self, entity: Entity, component: C)
    where
        C: Component
    {
        self.add_component(entity, component).unwrap();
    }
}

impl UiButtonBuilderTarget for CommandBuffer {
    fn create_entity(&mut self) -> Entity {
        self.insert((), Some(()))[0]
    }

    fn add_component<C>(&mut self, entity: Entity, component: C)
    where
        C: Component
    {
        self.add_component(entity, component);
    }
}

#[derive(Clone, Debug)]
pub struct UiButtonBuilder {
    x: f32,
    y: f32,
    z: f32,
    width: f32,
    height: f32,
    anchor: Anchor,
    pivot: Anchor,
    stretch: Stretch,
    text: String,
    text_color: Srgba,
    font: Option<Handle<FontAsset>>,
    font_size: f32,
    image: UiImage,
    parent: Option<Entity>,
    on_click_start: SmallVec<[UiButtonActionType; 2]>,
    on_click_stop: SmallVec<[UiButtonActionType; 2]>,
    on_hover_start: SmallVec<[UiButtonActionType; 2]>,
    on_hover_stop: SmallVec<[UiButtonActionType; 2]>,
}

impl Default for UiButtonBuilder {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: DEFAULT_Z,
            width: DEFAULT_WIDTH,
            height: DEFAULT_HEIGHT,
            anchor: Anchor::Middle,
            pivot: Anchor::Middle,
            stretch: Stretch::NoStretch,
            text: String::new(),
            text_color: Srgba::from_components(DEFAULT_TEXT_COLOR),
            font: None,
            font_size: DEFAULT_FONT_SIZE,
            image: UiImage::SolidColor(Srgba::from_components(DEFAULT_BACKGROUND_COLOR)),
            parent: None,
            on_click_start: SmallVec::new(),
            on_click_stop: SmallVec::new(),
            on_hover_start: SmallVec::new(),
            on_hover_stop: SmallVec::new(),
        }
    }
}

impl UiButtonBuilder {
    pub fn with_position(mut self, x: f32, y: f32) -> Self {
        self.x = x;
        self.y = y;
        self
    }

    pub fn with_layer(mut self, z: f32) -> Self {
        self.z = z;
        self
    }

    pub fn with_size(mut self, width: f32, height: f32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    pub fn with_anchor(mut self, anchor: Anchor) -> Self {
        self.anchor = anchor;
        self
    }

    pub fn with_pivot(mut self, pivot: Anchor) -> Self {
        self.pivot = pivot;
        self
    }

    pub fn with_stretch(mut self, stretch: Stretch) -> Self {
        self.stretch = stretch;
        self
    }

    pub fn with_text<S>(mut self, text: S) -> Self
    where
        S: ToString
    {
        self.text = text.to_string();
        self
    }

    pub fn with_text_color(mut self, text_color: Srgba) -> Self {
        self.text_color = text_color;
        self
    }

    pub fn with_hover_text_color(mut self, text_color: Srgba) -> Self {
        self.on_hover_start.push(UiButtonActionType::SetTextColor(text_color));
        self.on_hover_stop.push(UiButtonActionType::UnsetTextColor(text_color));
        self
    }

    pub fn with_press_text_color(mut self, text_color: Srgba) -> Self {
        self.on_click_start.push(UiButtonActionType::SetTextColor(text_color));
        self.on_click_stop.push(UiButtonActionType::UnsetTextColor(text_color));
        self
    }

    pub fn with_font(mut self, font: Handle<FontAsset>) -> Self {
        self.font = Some(font);
        self
    }

    pub fn with_font_size(mut self, font_size: f32) -> Self {
        self.font_size = font_size;
        self
    }

    pub fn with_image(mut self, image: UiImage) -> Self {
        self.image = image;
        self
    }

    pub fn with_hover_image(mut self, image: UiImage) -> Self {
        self.on_hover_start.push(UiButtonActionType::SetImage(image.clone()));
        self.on_hover_stop.push(UiButtonActionType::UnsetImage(image));
        self
    }

    pub fn with_press_image(mut self, image: UiImage) -> Self {
        self.on_click_start.push(UiButtonActionType::SetImage(image.clone()));
        self.on_click_stop.push(UiButtonActionType::UnsetImage(image));
        self
    }

    pub fn with_parent(mut self, parent: Entity) -> Self {
        self.parent = Some(parent);
        self
    }

    pub fn build<T>(self, target: &mut T) -> UiButton
    where
        T: UiButtonBuilderTarget
    {
        let entity = target.create_entity();

        target.add_component(
            entity,
            UiTransform::new(
                "PLACEHOLDER",
                self.anchor,
                self.pivot,
                self.x,
                self.y,
                self.z,
                self.width,
                self.height,
            ),
        );

        if let Some(parent) = self.parent {
            target.add_component(entity, parent);
        }

        target.add_component(entity, self.image);

        target.add_component(
            entity,
            UiText::new(
                self.font.expect("TODO: Implement default font"),
                self.text,
                self.text_color,
                self.font_size,
            ),
        );

        if !self.on_click_start.is_empty()
            || !self.on_click_stop.is_empty()
            || !self.on_hover_start.is_empty()
            || !self.on_hover_stop.is_empty()
        {
            target.add_component(
                entity,
                UiButtonActionRetrigger {
                    on_click_start: actions_with_target(
                        self.on_click_start.into_iter(),
                        entity,
                    ),
                    on_click_stop: actions_with_target(
                        self.on_click_stop.into_iter(),
                        entity,
                    ),
                    on_hover_start: actions_with_target(
                        self.on_hover_start.into_iter(),
                        entity,
                    ),
                    on_hover_stop: actions_with_target(
                        self.on_hover_stop.into_iter(),
                        entity,
                    ),
                },
            );
        }

        UiButton::new(entity)
    }
}

fn actions_with_target<I>(actions: I, target: Entity) -> Vec<UiButtonAction>
where
    I: Iterator<Item = UiButtonActionType>,
{
    actions.map(|action| UiButtonAction::new(target, action)).collect()
}