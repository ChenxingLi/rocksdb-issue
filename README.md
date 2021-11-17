# rocksdb-issue
A minimum version to reproduce the performance issue in `kvdb-rocksdb`. This issue affects v0.2 to the most recent version.

After writing a large amount of random data to `rocksdb`, the read operations cost 200 more time. 

The `rocksdb` is accessed via a rust wrapper `kvdb-rocksdb`.

## More details
The profiling shows that function `BlockFetcher::CheckBlockChecksum` in [rocksdb](https://github.com/facebook/rocksdb/) spends a lot of time. 

The confusing thing is the read option `verify_checksums` has been set to `false` in `kvdb-rocksdb`. Why `rocksdb` still spends a lot of time on verifying checksums?

Later, I found that the variable `read_options_` in `BlockFetcher::CheckBlockChecksum` is not from the user's input. It comes from a default `ReadOptions` (in which `verify_checksums` is `true`) made in function `FilterBlockReaderCommon<TBlocklike>::GetOrReadFilterBlock`. 

```C++
template <typename TBlocklike>
Status FilterBlockReaderCommon<TBlocklike>::GetOrReadFilterBlock(
    bool no_io, GetContext* get_context,
    BlockCacheLookupContext* lookup_context,
    CachableEntry<TBlocklike>* filter_block) const {
  assert(filter_block);

  if (!filter_block_.IsEmpty()) {
    filter_block->SetUnownedValue(filter_block_.GetValue());
    return Status::OK();
  }

  ReadOptions read_options;
  if (no_io) {
    read_options.read_tier = kBlockCacheTier;
  }

  return ReadFilterBlock(table_, nullptr /* prefetch_buffer */, read_options,
                         cache_filter_blocks(), get_context, lookup_context,
                         filter_block);
}
```

Here, a default `read_options` is made and the inner call will verify the checksum regardless of customized options. 
