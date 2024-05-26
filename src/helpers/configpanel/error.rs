#[derive(Clone, Debug, Snafu, PartialEq)]
pub enum ConfigPanelError {
    #[snafu(display("FilterKey cannot have so many depth levels (3 max): {filter_key}"))]
    FilterKeyTooDeep { filter_key: String },
    #[snafu(display("FilterKey cannot be empty"))]
    FilterKeyNone,
}
