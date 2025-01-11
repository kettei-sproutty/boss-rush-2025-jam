use crate::prelude::*;

#[derive(Component)]
pub struct StateOnPress<S: States> {
  pub action: S,
}

impl<S: States> StateOnPress<S> {
  pub fn from(state: S) -> Self {
    Self { action: state }
  }
}
