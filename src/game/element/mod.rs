use bevy::prelude::Component;

#[allow(dead_code)]
#[derive(Component)]
pub enum Element {
  Fire,
  Water,
  Electric,
  Earth,
}

impl Element {
  #[allow(dead_code)]
  pub fn damage_multiplier(&self, other: &Element) -> f32 {
    match (self, other) {
      (Element::Fire, Element::Water) => 0.5,
      (Element::Fire, Element::Electric) => 1.5,
      (Element::Fire, Element::Earth) => 1.0,
      (Element::Water, Element::Fire) => 1.5,
      (Element::Water, Element::Electric) => 1.0,
      (Element::Water, Element::Earth) => 0.5,
      (Element::Electric, Element::Fire) => 0.5,
      (Element::Electric, Element::Water) => 1.5,
      (Element::Electric, Element::Earth) => 1.0,
      (Element::Earth, Element::Fire) => 1.0,
      (Element::Earth, Element::Water) => 1.5,
      (Element::Earth, Element::Electric) => 0.5,
      _ => 1.0,
    }
  }
}
