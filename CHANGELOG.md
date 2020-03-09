# Changelog

## 0.4.0

- remove `fn Texture::clear_depth`, use `Context::clear_depth(&mut texture)` instead.
- add required trait method `fn DrawTarget::receive_clear_depth`.
- add `fn Context::clear_depth`.
- reduce the requires OpenGL version to **3.2**.

## 0.3.2

- fix `docs.rs` package metadata.

## 0.3.1

- add feature `serde1`.

## 0.3.0

- rename `Context::draw_line` to `Context::debug_line`.
- add `fn Context::debug_rectangle`.
- add required trait method `fn DrawTarget::receive_rectangle`.

### 0.2.2

- improve draw_line performance

### 0.2.1

- improve docs + refactor

## 0.2.0

- add `fn Texture::from_image`
- export the `image` crate.
- add `fn Context::draw_line`.
- add required trait method `fn DrawTarget::receive_line`.
- update error type of `fn Texture::load`.

## 0.1.0

initial release