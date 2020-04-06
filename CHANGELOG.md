# Changelog

## 0.7.1

- implement `error::Error` and `fmt::Display` for all error types.

## 0.7.0

- remove `fn Context::take_screenshot`.
- remove `fn Texture::get_image_data`.
- add required trait method `fn DrawTarget::get_image_data`.
- add `fn Context::image_data` which can be used on all `DrawTarget`s.

## 0.6.0

- `fn Context::window_surface` was renamed to `fn Context::surface` and may now panic if used incorrectly.
- `fn Context::finalize_frame` was changed to `fn Context::present(surface: WindowSurface)`.
- add logging using the `log` crate.
- update `glutin` to version `0.24`

## 0.5.1

- add basic support for HiDPI.

## 0.5.0

- change `fn Context::new` to not require an `EventsLoop` as an argument.
- fix bug for OpenGL version not supporting `ARB_framebuffer_no_attachments`.

## 0.4.0

- implement actual error types and update function return types.
- add `fn Offset::into_inner` and `fn Scaled::into_inner`.
- update image from version `0.22` to `0.23`.
- remove `fn Texture::clear_depth`, use `Context::clear_depth(&mut texture)` instead.
- add required trait method `fn DrawTarget::receive_clear_depth`.
- add `fn Context::clear_depth`.
- add `fn Context::maximum_texture_size`
- update error type of `fn Texture::new`.
- reduce the required OpenGL version to **3.2**.

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
