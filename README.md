# recycle_vec

Note: There used to be an [RFC](https://github.com/rust-lang/rfcs/pull/2802)
for making this functionality a part of the standard library. However,
it was closed because 1) an additional API as little as this
doesn't need a full RFC anymore 2) but the capabilities of the
language around checking invariants statically aren't there yet,
so the API would be suboptimal and therefore it's best to wait for
the const generics and static checking stuff to catch up.

This crate provides a `recycle` extension method for `Vec`.
It's intended to change the type of the `Vec` while "recycling"
the underlying allocation. This is a trick that is useful especially
when storing data with short lifetimes in `Vec`:
```
   　let mut objects: Vec<Object<'static>> = Vec::new();

  　 while let Some(byte_chunk) = stream.next() { // byte_chunk only lives this scope
       　let mut objects_temp: Vec<Object<'_>> = objects.recycle();

      　 // Zero-copy parsing; Object has references to chunk
      　 deserialize(byte_chunk, &mut objects_temp)?;
     　  process(&objects_temp)?;

      　 objects = objects_temp.recycle();
 　  } // byte_chunk lifetime ends
```
## Notes about safety
This crate uses internally `unsafe` to achieve it's functionality.
However, it provides a safe interface. To achieve safety, it does
the following precautions:
1. It truncates the `Vec` to zero length, dropping all the values.
This ensures that no values of arbitrary types are transmuted
accidentally.
2. It checks that the sizes and alignments of the source and target
types match. This ensures that the underlying block of memory backing
`Vec` is compatible layout-wise.
3. It creates a new `Vec` value using `from_raw_parts` instead of
transmuting, an operation whose soundness would be questionable.
