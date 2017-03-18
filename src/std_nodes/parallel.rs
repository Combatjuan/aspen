//! Nodes that tick their children in parallel
use std::sync::Arc;
use node::{Node, Internals, IdType};
use status::Status;

/// Implements a standard Parallel node
pub struct Parallel<T: Send + Sync + 'static>
{
	/// Children to be ticked
	children: Vec<Node<T>>,

	/// Number of nodes required to succeed before this one does
	required_successes: usize,
}
impl<T: Send + Sync + 'static> Parallel<T>
{
	/// Creates a new Parallel node
	pub fn new(children: Vec<Node<T>>, required_successes: usize) -> Node<T>
	{
		let internals = Parallel {
			children: children,
			required_successes: required_successes,
		};
		Node::new(internals)
	}
}
impl<T: Send + Sync + 'static> Internals<T> for Parallel<T>
{
	fn tick(&mut self, world: &Arc<T>) -> Status
	{
		let mut successes = 0;
		let mut failures = 0;

		// Tick every single child node
		for child in self.children.iter_mut() {
			let child_status = child.tick(world);

			if child_status == Status::Succeeded {
				successes += 1;
			}
			else if child_status == Status::Failed {
				failures += 1;
			}
		}

		// Return a result based on the children
		if successes >= self.required_successes {
			// Enough children succeeded
			Status::Succeeded
		} else if failures > (self.children.len() - self.required_successes) {
			// Too many children failed - it is impossible to succeed
			Status::Failed
		} else {
			// Status is still undetermined
			Status::Running
		}
	}

	fn reset(&mut self)
	{
		// Reset all of our children
		for child in self.children.iter_mut() {
			child.reset();
		}
	}

	fn children(&self) -> Vec<&Node<T>>
	{
		self.children.iter().collect()
	}

	fn children_ids(&self) -> Vec<IdType>
	{
		self.children.iter().map(|c| c.id()).collect()
	}

	fn type_name(&self) -> &'static str
	{
		"Parallel"
	}
}

#[cfg(test)]
mod test
{
	use std::sync::Arc;
	use std::sync::atomic::AtomicBool;
	use node::Node;
	use status::Status;
	use std_nodes::*;

	#[test]
	fn success()
	{
		let world = Arc::new(AtomicBool::new(true));
		let mut children = vec![YesTick::new(Status::Succeeded),
		                        YesTick::new(Status::Succeeded),
		                        YesTick::new(Status::Running),
		                        YesTick::new(Status::Running),
		                        YesTick::new(Status::Failed),
		                        YesTick::new(Status::Failed)];
		let mut parallel = Parallel::new(children, 2);
		let status = parallel.tick(&world);
		drop(parallel);
		assert_eq!(status, Status::Succeeded);
	}

	#[test]
	fn failure()
	{
		let world = Arc::new(AtomicBool::new(true));
		let children = vec![YesTick::new(Status::Succeeded),
		                    YesTick::new(Status::Succeeded),
		                    YesTick::new(Status::Running),
		                    YesTick::new(Status::Running),
		                    YesTick::new(Status::Failed),
		                    YesTick::new(Status::Failed)];
		let mut parallel = Parallel::new(children, 5);
		let status = parallel.tick(&world);
		drop(parallel);
		assert_eq!(status, Status::Failed);
	}

	#[test]
	fn running()
	{
		let world = Arc::new(AtomicBool::new(true));
		let children = vec![YesTick::new(Status::Succeeded),
		                    YesTick::new(Status::Succeeded),
		                    YesTick::new(Status::Running),
		                    YesTick::new(Status::Running),
		                    YesTick::new(Status::Failed),
		                    YesTick::new(Status::Failed)];
		let mut parallel = Parallel::new(children, 3);
		let status = parallel.tick(&world);
		drop(parallel);
		assert_eq!(status, Status::Running);
	}
}
