# `bevy-webp-anim`

Plugin for loading animated webp images in bevy.

//! We assume the files are animations.
//! The files are later on parsed by the [`image`] crate.

TODO:

- [ ] Improve framerate settings
- [ ] Improve loader settings
- [ ] Threadpool for frame decoding
- [ ] Reuse decoded frames between assets
- [ ] Docs and examples

// 2. multiple independent decoders
// 5. threadpool is a resource itself? that way you could drop it.
// 7. it holds receivers decoders who run on async channel sender
// 8. the async channel sleeps until it can send the next frame.
// 10. configure the number of threads when spawning the threadpool (or even
// change it runtime)

```
// /// Clone this to start reading from the beginning of the byte buffer.
// struct RepeatableReader {
//     from: Arc<Vec<u8>>,
//     last_read_to_index_exclusive: usize,
// }

// impl Clone for RepeatableReader {
//     /// Start reading from the beginning of the byte buffer.
//     fn clone(&self) -> Self {
//         Self {
//             from: Arc::clone(&self.from),
//             last_read_to_index_exclusive: 0,
//         }
//     }
// }

// impl std::io::Read for RepeatableReader {
//     fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
//         if buf.is_empty() {
//             return Ok(0);
//         }

//         let total_bytes = self.from.len();

//         if self.last_read_to_index_exclusive >= total_bytes {
//             Ok(0)
//         } else {
//             let read_until_index_exclusive =
//                 self.last_read_to_index_exclusive +
// buf.len().min(total_bytes);             let bytes = &self.from
//
// [self.last_read_to_index_exclusive..read_until_index_exclusive];

//             buf[..bytes.len()].copy_from_slice(bytes);

//             Ok(bytes.len())
//         }
//     }
// }
```

    // TODO: Enable an alternative with lower memory usage and
    // faster startup times where we decode each frame every time
    // it is played.
