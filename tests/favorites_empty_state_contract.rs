const FAVORITES: &str = include_str!("../src/render_builtins/favorites.rs");

#[test]
fn favorites_empty_state_copy_is_modeled() {
    assert!(
        FAVORITES.contains("enum FavoritesEmptyState")
            && FAVORITES.contains("NoFavoritesYet")
            && FAVORITES.contains("NoFilteredMatches"),
        "Favorites empty-state copy should use named states"
    );
    assert!(
        FAVORITES.contains("fn from_filter(filter: &str) -> Self")
            && FAVORITES.contains("fn message(self) -> &'static str"),
        "Favorites empty states should own filter classification and visible copy"
    );
    assert!(
        FAVORITES.contains("FavoritesEmptyState::from_filter(&filter).message()"),
        "Favorites renderer should derive empty-state copy from the model"
    );
    assert!(
        !FAVORITES.contains("child(if filter.is_empty()"),
        "Favorites empty-state copy must not regress to inline filter-empty branching"
    );
}
