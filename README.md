# Iced Texture Canvas (WIP)

A widget similar to a [`image::Viewer`](https://docs.iced.rs/iced/widget/image/viewer/index.html) that lets you view bitmap data but with added enhancements.

What sets this apart is that you can freely modify the image data without re-allocating or resorting to locks. This is good if you need to display frequently changing image data.

Instead of using [`image::Handle`](https://docs.iced.rs/iced/advanced/image/enum.Handle.html), this crate provides `iced_texture_canvas::Bitmap` an rgba buffer stored on the CPU.

And to view that buffer, you use `iced_texture_canvas::texture_canvas`.


```rust
use iced_texture_canvas::{bitmap, texture_canvas};

// create your bitmap image
let mut bitmap = bitmap(500, 500);

// fill it with color
bitmap.buffer_mut().fill(0xffffffff);


// display it in your view method
texture_canvas(&bitmap)

```

The api also takes a few inspirations from [`MouseArea`](https://docs.iced.rs/iced/widget/struct.MouseArea.html)
<!-- 
## Advanced Usage
### SurfaceHandler and Surface -->

# Run Example

```
cargo run -p demo
```


# Todos
* API improvements
* Explore abstracting over image formats instead of just rgba.
* Use a texture atlas for efficiently drawing multiple textures.
* Layering + overlay support in canvas space.
* A static viewer analogous to the image widget.

# Limitations
* Only works if you're using the wgpu renderer.