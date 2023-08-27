use std::{rc::{Rc, Weak}, borrow::{Borrow, BorrowMut}, collections::{VecDeque, HashSet}, cell::{Cell, RefCell}, default, ops::{Add, Sub, Neg, Index}, };


mod graph;


//    impl<'a : 'b, 'b, P, A, W : Default> graph::Bipartite<P, A, W>
// where graph::Edge<P, A, W> : HasWeight<'b, Weight<'a> = W> + PartialEq,
//     graph::PNode<P, A, W> : HasCapacity,
//     W : Ord + Add + Sub + Neg<Output = W>,
// {
//     //todo : constructor, add, remove vertices/edges.

//     fn max_match(&mut self) -> Edges<P, A, W> {
//         let mut otws : VecDeque<   Rc<  RefCell< graph::ANode<P, A, W> >  >   > =
//             self.applicants.iter()
//             .filter(|a|RefCell::borrow(a).open_to_work.get())
//             .map(Rc::clone).collect();

//         let g_sorted = Rc::new(true);

//         fn check_sorted<'a : 'b, 'b, P, A, W : Ord>
//             (edges : &mut Edges<P, A, W>, sorted : &mut Weak<bool>, sorted_p : &mut usize, g_sorted: &Rc<bool>)
//         where graph::Edge<P, A, W> : HasWeight<'b, Weight<'a> = W> + PartialEq,
//             W : Ord + Neg<Output = W>,
//         {
//             if !sorted.upgrade().is_some_and(|b| *b) {
//                 let current_pairing = edges[*sorted_p].upgrade()
//                     .and_then(|e| {
//                         e.a_i.upgrade()?;
//                         e.a_i.upgrade()?;
//                         Some(e)
//                     });
                
//                 edges.retain(|e| e.upgrade().is_some());
//                 edges.sort_by_key(|e| - e.upgrade().unwrap().weight());

//                 *sorted_p = current_pairing.and_then(|pairing|
//                     edges.iter().position(|e| e.upgrade().unwrap() == pairing)
//                 ).unwrap_or(0);

//                 *sorted = Rc::<bool>::downgrade(g_sorted);
//             };
//         }
//         fn salary<'a : 'b, 'b, P, A, W : Default>(e: Option<  Weak< graph::Edge<P, A, W> >  >)
//         where graph::Edge<P, A, W> : HasWeight<'b, Weight<'a> = W>,
//         {
//             (|| -> Option<W> { Some(e?.upgrade()?.weight()) })()
//             .unwrap_or_else(W::default);
//         }

//         while let Some(otw) = otws.pop_front() {
//             let mut otw = RefCell::borrow_mut(&otw);
//             let current_pay = salary(otw.current_post);

//             check_sorted(&mut otw.neighbours, &mut otw.sorted, &mut otw.neigh_index, &g_sorted);
//             for job in otw.neighbours.iter().flat_map(Weak::upgrade) {
//                 let Some(post) = job.p_node.upgrade() else{ continue; };
//                 let mut post = RefCell::borrow_mut(&post);
//                 if post.current_applicants.len() < post.capacity() {

//                 };
                
//                 check_sorted(&mut post.edges, &mut post.sorted, &mut post.sorted_p, &g_sorted);

//                 for competitor in post.current_applicants.iter().flat_map(Weak::upgrade) {
//                     let Some(applicant) = competitor.a_node.upgrade() else{ continue; };
//                     let mut applicant = RefCell::borrow_mut(&applicant);

//                 }
//             }
//         }
//         todo!()
//     }
// }


#[cfg(test)]
mod test{
    use std::{rc::Weak, thread::LocalKey};

    fn test_max_match() {
        
    }
}
