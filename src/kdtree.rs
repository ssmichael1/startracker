pub trait NodeElement: Clone + std::fmt::Debug {
    const NDIM: usize;

    fn compare_on_dim(&self, other: &Self, dim: usize) -> std::cmp::Ordering;
    fn distance(&self, other: &Self) -> f64;
    fn distance_on_dim(&self, other: &Self, dim: usize) -> f64;
}

#[derive(Clone, Debug)]
struct Node<T>
where
    T: NodeElement,
{
    data: T,
    left: Option<Box<Node<T>>>,
    right: Option<Box<Node<T>>>,
}

impl<T> Node<T>
where
    T: NodeElement,
{
    fn new(data: T) -> Self {
        Node {
            data,
            left: None,
            right: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct KdTree<N>
where
    N: NodeElement,
{
    root: Option<Box<Node<N>>>,
}

impl<N> KdTree<N>
where
    N: NodeElement,
{
    fn compute_sortidx(data: &Vec<N>, dim: usize) -> Vec<usize> {
        let mut indices: Vec<usize> = (0..data.len()).collect();
        indices.sort_by(|&a, &b| data[a].compare_on_dim(&data[b], dim));
        indices
    }

    pub fn create_from_vector(data: &Vec<N>) -> Self {
        let node = KdTree::<N>::create_node(data, 0);
        KdTree::<N> { root: node }
    }

    fn create_node(data: &Vec<N>, depth: usize) -> Option<Box<Node<N>>> {
        if data.is_empty() {
            return None;
        }

        let dim = depth % N::NDIM;
        let indices = KdTree::<N>::compute_sortidx(data, dim);
        let median_idx: usize = data.len() / 2;
        let median = indices[median_idx];
        let mut node = Node::new(data[median].clone());

        if median_idx > 0 {
            let left_data: Vec<N> = indices[0..median_idx]
                .iter()
                .map(|&idx| data[idx].clone())
                .collect();
            let left_node = KdTree::<N>::create_node(&left_data, depth + 1);
            node.left = left_node;
        }
        if median_idx < data.len() - 1 {
            let right_data: Vec<N> = indices[(median_idx + 1)..]
                .iter()
                .map(|&idx| data[idx].clone())
                .collect();
            let right_node = KdTree::<N>::create_node(&right_data, depth + 1);
            node.right = right_node;
        }
        Some(Box::new(node))
    }

    pub fn points_in_range(&self, point: &N, range: f64) -> Vec<&N> {
        let mut result = Vec::new();
        if let Some(ref root) = self.root {
            KdTree::<N>::points_in_range_recursive(&root, point, range, 0, &mut result);
        }
        result
    }

    fn points_in_range_recursive<'a>(
        node: &'a Box<Node<N>>,
        point: &N,
        range: f64,
        depth: usize,
        result: &mut Vec<&'a N>,
    ) {
        let dim = depth % N::NDIM;
        let dist = node.data.distance(point);
        if dist <= range {
            result.push(&node.data);
        }

        if let Some(ref left) = node.left {
            if point.distance_on_dim(&left.data, dim) <= range {
                KdTree::<N>::points_in_range_recursive(left, point, range, depth + 1, result);
            }
        }
        if let Some(ref right) = node.right {
            if point.distance_on_dim(&right.data, dim) <= range {
                KdTree::<N>::points_in_range_recursive(right, point, range, depth + 1, result);
            }
        }
    }
}

impl<const D: usize> NodeElement for [f64; D] {
    const NDIM: usize = D;

    fn compare_on_dim(&self, other: &Self, dim: usize) -> std::cmp::Ordering {
        self[dim].partial_cmp(&other[dim]).unwrap()
    }

    fn distance(&self, other: &Self) -> f64 {
        self.iter()
            .zip(other.iter())
            .map(|(a, b)| (a - b).powi(2))
            .sum::<f64>()
            .sqrt()
    }

    fn distance_on_dim(&self, other: &Self, dim: usize) -> f64 {
        (self[dim] - other[dim]).abs()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_kdtree() {
        let data = vec![
            [2.0, 3.0],
            [5.0, 4.0],
            [9.0, 6.0],
            [4.0, 7.0],
            [8.0, 1.0],
            [7.0, 2.0],
        ];

        let kdtree = KdTree::<[f64; 2]>::create_from_vector(&data);
        println!("created tree\n");

        let point = [9.0, 2.0];
        let ranges = data
            .iter()
            .map(|x| x.distance(&point))
            .collect::<Vec<f64>>();
        println!("ranges = {:?}", ranges);

        let range = 4.1;
        let result = kdtree.points_in_range(&point, range);
        println!("result = {:?}", result);
        //assert_eq!(result.len(), 2);
    }
}
