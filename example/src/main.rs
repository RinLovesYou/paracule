use libflipnote::ppm::file::PPMFile;

fn main() {
    // Load a PPM file
    let ppm_file = PPMFile::from_path("/home/sarah/Downloads/bokeh.ppm").unwrap();

    // Save the thumbnail as a PNG file
    ppm_file.thumbnail.save_image_as("/home/sarah/Pictures/bokeh_thumbnail.png").unwrap();

    // Save the PPM file as a new file
    ppm_file.save_as("/home/sarah/Pictures/bokeh.ppm").unwrap();
}
