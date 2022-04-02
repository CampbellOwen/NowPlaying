use image::{GenericImageView, Pixel};
use std::collections::HashSet;

#[derive(Debug)]
pub struct Cluster<T: Pixel> {
    pub averagePixel: T,
    pub members: HashSet<T>,
}

pub fn cluster<I: GenericImageView>(img: &I, num_clusters: u32) -> Vec<Cluster<I::Pixel>>
where
    I::Pixel: 'static,
    <I::Pixel as Pixel>::Subpixel: 'static,
{
    vec![Cluster {
        averagePixel: img.get_pixel(10, 10).clone(),
        members: HashSet::new(),
    }]
}
