use poem_openapi::Tags;

#[derive(Debug, Tags)]
pub enum ApiTags {
    Health,
    Products,
    ShoppingItems,
    Suggestions,
}
