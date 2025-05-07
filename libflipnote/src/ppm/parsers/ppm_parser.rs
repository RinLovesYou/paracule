use binrw::BinResult;

#[binrw::parser(reader)]
pub fn ppm_parser() -> BinResult<Vec<u8>> {
    reader.seek(std::io::SeekFrom::Start(0))?;
    let mut data = vec![];
    reader.read_to_end(&mut data)?;

    reader.seek(std::io::SeekFrom::Start(4))?;

    Ok(data)
}
