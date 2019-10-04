/// All the errors this crate can throw
#[derive(Debug)]
pub enum Error {
    /// Could not open the given file while loading a memory
    CouldNotOpenFile(std::io::Error),

    /// Could not create the given file while saving a memory
    CouldNotCreateFile(std::io::Error),

    /// Could not read the file as a valid zip
    CouldNotReadZip(zip::result::ZipError),

    /// Could not find a valid file in a given zip
    CouldNotReadFirstFile(zip::result::ZipError),

    /// Could not create a zip entry while saving a memory
    CouldNotCreateZipEntry(zip::result::ZipError),

    /// Deserialize error while loading a memory, should never occur
    CouldNotDeserialize(bincode::Error),

    /// Serialize error while saving a memory, should never occur
    CouldNotSerialize(bincode::Error),
}
