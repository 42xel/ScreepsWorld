use std::iter::Map;

/// A weighted bipartite graph with one set (the jobs) having a capacity potentially != 1 for max-matching.
/// It's equivalent to duplicating the nodes, but more effeicient and nicer to interface for my particular use case.
#[derive(Default)]
pub (super) struct Bipartite<P, A, W> {
    posts : Vec< PNode<P> >,
    applicants : Vec< ANode<A> >,
    edges : Vec< Edge<W> >,
}

type PostIndex = usize;
type ApplIndex = usize;
type EdgeIndex = usize;
type NeighIndex = usize;

struct ANode<A> {
    // static graph data
    applicant : A,
    neighbours : Vec<EdgeIndex>,

    // data about the current pseudo-flow
    current_post : Option<NeighIndex>,

    // annotation for the algorithm
    open_to_work : bool,
}
impl<A> ANode<A> {
    fn new(applicant: A) -> Self {
        Self { applicant,
            neighbours : vec![],
            current_post : None,
            open_to_work : true,
        }
    }
}

struct PNode<P> {
    // static graph data
    post : P,
    neighbours : Vec<EdgeIndex>,
    // max neighbourhood size
    capacity : NeighIndex,

    // data about the current pseudo-flow
    current_applicants : Vec<NeighIndex>,
}
impl<P> PNode<P> {
    fn new(post : P, capacity : NeighIndex) -> Self {
        Self { post,
            neighbours : vec![],
            capacity,
            current_applicants : vec![],
        }
    }
}

struct Edge<W>
{
    p_i : PostIndex,
    a_i : ApplIndex,
    weight : W,
}
impl<W> Edge<W> {
    fn new(p_i: PostIndex, a_i: ApplIndex, weight: W) -> Self { Self { p_i, a_i, weight } }
}

impl<P, A, W> Bipartite<P, A, W> {
    fn new() -> Self {
        Self {
            posts : vec![],
            applicants : vec![],
            edges : vec![],
        }
    }
}

trait GraphElementMarker {}
struct GraphElementEdge {} impl GraphElementMarker for GraphElementEdge {}
struct GraphElementVertex0 {} impl GraphElementMarker for GraphElementVertex0 {}
struct GraphElementVertex1 {} impl GraphElementMarker for GraphElementVertex1 {}

/// Extend the bipartite graph with the contents of an iterator.
///
/// It is similar to the [`Extend`] trait, with  the following difference :
/// - it has a trait marker to specify what is being extended (vertices or edges).
/// - it returns an iterator with the indices of the element built to extend the graph, for chaining purposes.
pub trait ExtendGraph< ElemType : GraphElementMarker, A> {
    /// Which kind of iterator are we yielding ?
    type IterOut<IterIn: IntoIterator<Item = A>> : Iterator<Item = usize> + ?Sized = dyn Iterator<Item = usize>;

    fn extend<IterIn: IntoIterator<Item = A>>(&mut self, iter: IterIn) -> Self::IterOut<IterIn>;

    fn extend_one(&mut self, item: A) -> usize {
        self.extend(Some(item)).last().unwrap()
    }

    fn extend_reserve(&mut self, additional: usize) {
        let _ = additional;
    }
}

impl<P, A, W> ExtendGraph<GraphElementEdge, (usize, usize, W)> for Bipartite<P, A, W>
{
    //type IterOut<IterIn: IntoIterator<Item = (usize, usize, W)>> = dyn Iterator<Item = usize>;// Map< IterIn, dyn FnMut(IterIn::Item) -> usize > ;

    fn extend<IterIn: IntoIterator<Item = (usize, usize, W)>>(&mut self, iter: IterIn) -> Box< Self::IterOut<IterIn> > {
        iter.into_iter().map(|(p_i, a_i, weight)| {
            let e = Edge::new(p_i, a_i, weight);
            let e_i = self.edges.len();

            self.edges.push(e);
            self.posts[p_i].neighbours.push(e_i);
            let mut appl = self.applicants[a_i];
            appl.neighbours.push(e_i);
            appl.open_to_work = true;

            e_i
        })
    }
    //type IterOut<IterIn: IntoIterator<Item = A>> = Map<IterIn, FnMut>;

    // fn extend<IterIn : IntoIterator<Item = (usize, usize, W)>>(&mut self, iter: IterIn) {
    // }
}

// impl<P, A, W, Ip> Extend<(A, Ip)> for Bipartite<P, A, W>
// where Ip : Iterator<    Item = Rc<  RefCell< PNode<P, A, W> >  >    >,
// {
//     fn extend<T: IntoIterator<Item = (A, Ip)>>(&mut self, iter: T) {
//         for (applicant, iter_p) in iter {
//             let mut a_node = Rc::new(RefCell::new(self::ANode::new(applicant)));
//             let mut a_node_ref = RefCell::borrow(&a_node);

//             for p_node in iter_p {
//                 let e = Rc::new(Edge::new(&p_node, &a_node));
//                 RefCell::borrow(&p_node).neighbours.push(Rc::downgrade(&e));
//                 a_node_ref.neighbours.push(Rc::downgrade(&e));
            
//                 self.edges.push(e);
//             }
        
//             a_node_ref.open_to_work.set(true);
//             self.applicants.push(a_node);
//         }
//     }
// }



// TODO : Make a simple static structure and algo,
// and only later make traits and getter for dynamic (but tick cached) capacity and weights.

// /// trait for dynamic capacity
// trait HasCapacity { fn capacity(&self) -> usize ; }

// /// traits for dynamic weighting
// trait HasWeight<'b> {
//     type Weight<'a : 'b>;
//     fn weight<'a : 'b>(&self) -> Self::Weight<'a>;
// }
// trait GetWeightBy<'b> {
//     type Weight<'a : 'b>;
//     /// a potentially expensive or dynamic way to get the weight
//     fn calc_weight<'a : 'b>(&self) -> Self::Weight<'a>;
// }

// /// a trait for when the weight can become stale (eg. at the end of each tick, after a move, etc.)
// trait HasWeakWeight<'b> : GetWeightBy<'b> {
//     fn weight_set<'a : 'b>(&self, value: Self::Weight<'a>);
//     fn weight_get<'a : 'b>(&self) -> Option<Self::Weight<'a>>;
// }

// impl< 'b, T : HasWeakWeight<'b> > HasWeight<'b> for T {
//     type Weight<'a : 'b> = < Self as GetWeightBy<'b> >::Weight<'a>;
//     fn weight<'a : 'b>(&self) -> Self::Weight<'a> {
//         self.weight_get().unwrap_or_else(|| {
//             let r = self.calc_weight();
//             self.weight_set(r);
//             r
//         })
//     }
// }
