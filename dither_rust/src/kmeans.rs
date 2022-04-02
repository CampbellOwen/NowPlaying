use image::{GenericImageView, Pixel};

#[derive(Debug)]
pub struct Cluster<T: Pixel> {
    pub average_pixel: T,
    pub members: Vec<T>,
}

pub fn pixel_difference<P: Pixel>(first: &P, second: &P) -> u32 {
    unimplemented!()
}

pub fn average_pixel<P: Pixel>(pixels: &[P]) -> P {
    unimplemented!()
}

pub fn cluster<I: GenericImageView>(img: &I, num_clusters: u32) -> Vec<Cluster<I::Pixel>>
where
    I::Pixel: 'static,
    <I::Pixel as Pixel>::Subpixel: 'static,
{
    let mut clusters: Vec<Cluster<I::Pixel>> = img
        .pixels()
        .take(num_clusters as usize)
        .map(|(_, _, pixel)| Cluster {
            average_pixel: pixel.clone(),
            members: vec![pixel.clone()],
        })
        .collect();

    let pixels = img.pixels().skip(num_clusters as usize);

    pixels.for_each(|(_, _, pixel)| {
        let (cluster_idx, score) = clusters
            .iter()
            .enumerate()
            .map(|(idx, cluster)| (idx, pixel_difference(&pixel, &cluster.average_pixel)))
            .min_by(|(_, s1), (_, s2)| s1.cmp(s2))
            .expect("There should always be a best cluster");

        clusters[cluster_idx].members.push(pixel);
        //clusters[cluster_idx].average_pixel = average_pixel(&clusters[cluster_idx].members);
    });

    clusters
        .iter_mut()
        .for_each(|cluster| cluster.average_pixel = average_pixel(&cluster.members));

    clusters
}
