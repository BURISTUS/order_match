use thiserror::Error;

#[derive(Debug, Clone, Error, PartialEq)]
pub enum ClientErrors {
    #[error("Unable to parse client id")]
    ParseClientIdError,
    #[error("Unable to parse client asset balance")]
    ParseAssetBalancesError,
    #[error("Unable to parse client dollar balance")]
    ParseDollarBalanceError,
    #[error("Can't parse client item: Insufficent input data")]
    ParseInsufficentInput,
}

#[derive(Clone, Debug, Error)]
pub enum GeneralErrors {
    #[error("Unable to read file")]
    ReadFileError,
    #[error("Unable to write file")]
    WriteFileError,
    #[error("Unable to get configuration")]
    GetConfigError,
    #[error("Error during file creation")]
    FileCreationError,
    #[error("Your dollar balance can't be negative")]
    NotEnaughDollars,
    #[error("Your asset balance can't be negative")]
    NotEnaughAsset,
    #[error("No such operation")]
    NoSuchOperationError,
    #[error("Unable to get a client from the map")]
    GetClientError,
    #[error("Unable to get an order from the map")]
    GetOrderError,
    #[error("Unable to get asset from the map")]
    GetAssetError,
}

#[derive(Error, Debug, Clone, PartialEq)]
pub enum OrderErrors {
    #[error("Unable to parse client Id")]
    ParseClientIdError,
    #[error("Unable to parse operation symbol")]
    ParseOperationError,
    #[error("Unable to parse asset")]
    ParseSymbolError,
    #[error("Unable to parse price")]
    ParseItemPriceError,
    #[error("Unable to parse volume")]
    ParseItemVolumeError,
    #[error("Can't parse order item: Insufficent input data")]
    ParseInsufficentInputError,
    #[error("No such operation symbol")]
    NoSuchOperationSymbolError,
}