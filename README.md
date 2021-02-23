# b_plus_tree
B+Tree that totally from scratch written with Rust lang.   

## Available operations
Basic operations common to BTreeMap in the Rust standard library

- get 
    ```rust:
    fn get<Q: ?Sized>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q> + Ord,
        Q: Ord, 
    ```
- insert
    ```rust:
    fn insert(&mut self, key: K, value: V) -> Option<V>
    ```
- remove
    ```rust:
    fn remove(&mut self, key: &K) -> Option<V>
    ```
- range
    ```rust:
    fn range<T: ?Sized, R>(&self, range: R) -> Range<'_, K, V>
    where
        T: Ord,
        K: Ord + Borrow<T>,
        R: RangeBounds<T>,
    ```
- keys
    ```rust:
    fn keys(&self) -> Keys<'_, K, V>
    ```
- values
    ```rust:
    fn values(&self) -> Values<'_, K, V>
    ```

and there're other things.

### License
MIT
