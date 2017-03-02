//! Nodes that have children and tick them in a sequential order as long as they
//! succeed.
//!
//! NOTE: There is no Sequence* node, since the choice of not having the nodes
//! automatically reset causes a normal Sequence node to have the same behavior
//! as a Sequence*.
use std::sync::Arc;
use node::Node;
use status::Status;

/// Implements a Sequence node
///
/// This node will tick all of its children in order until one of them returns
/// either `Status::Running` or `Status::Failed`. If none do, this node succeeds.
pub struct Sequence<T: Send + Sync + 'static>
{
	/// Vector containing the children of this node
	children: Vec<Box<Node<T>>>,
}
impl<T: Send + Sync + 'static> Sequence<T>
{
	/// Creates a new Sequence node from a vector of Nodes
	pub fn new(children: Vec<Box<Node<T>>>) -> Sequence<T>
	{
		Sequence {
			children: children,
		}
	}
}
impl<T: Send + Sync + 'static> Node<T> for Sequence<T>
{
	fn tick(&mut self, world: &Arc<T>) -> Status
	{
		// Tick the children in order
		for ptr in self.children.iter_mut() {
			// First, tick the current child to get its status
			let child_status = (*ptr).tick(world);

			// Then decide if we're done ticking based on our children
			if child_status != Status::Succeeded {
				return child_status;
			}
		}

		// All children succeeded
		Status::Succeeded
	}

	fn reset(&mut self)
	{
		// Reset all of our children
		for ptr in self.children.iter_mut() {
			(*ptr).reset();
		}
	}

	fn status(&self) -> Status
	{
		// See what the status of all the children are
		for ptr in self.children.iter() {
			// First, tick the current child to get its status
			let child_status = (*ptr).status();

			// Then decide if we're done ticking based on our children
			if child_status != Status::Succeeded {
				return child_status;
			}
		}

		// All children succeeded
		Status::Succeeded
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
	fn check_running()
	{
		// Use an atomic as the world (doesn't actually get used)
		let world = Arc::new(AtomicBool::new(true));

		// Set up the nodes
		let succeed = Box::new(YesTick::new(Status::Succeeded));
		let running = Box::new(YesTick::new(Status::Running));
		let err = Box::new(NoTick::new());

		// Put them all in a vector
		let mut children: Vec<Box<Node<AtomicBool>>> = Vec::new();
		children.push(succeed);
		children.push(running);
		children.push(err);

		// Add them to a sequence node
		let mut seq = Sequence::new(children);

		// Tick the sequence
		let status = seq.tick(&world);

		// Drop the sequence so the nodes can do their own checks
		drop(seq);

		// Make sure we got the expected value
		assert_eq!(status, Status::Running);
	}

	#[test]
	fn check_success()
	{
		// Use an atomic as the world (doesn't actually get used)
		let world = Arc::new(AtomicBool::new(true));

		// Set up the nodes
		let first = Box::new(YesTick::new(Status::Succeeded));
		let second = Box::new(YesTick::new(Status::Succeeded));

		// Put them all in a vector
		let mut children: Vec<Box<Node<AtomicBool>>> = Vec::new();
		children.push(first);
		children.push(second);

		// Add them to a sequence node
		let mut seq = Sequence::new(children);

		// Tick the sequence
		let status = seq.tick(&world);

		// Drop the sequence so the nodes can do their own checks
		drop(seq);

		// Make sure we got the expected value
		assert_eq!(status, Status::Succeeded);
	}

	#[test]
	fn check_fail()
	{
		// Use an atomic as the world (doesn't actually get used)
		let world = Arc::new(AtomicBool::new(true));

		// Set up the nodes
		let succeed = Box::new(YesTick::new(Status::Succeeded));
		let fail = Box::new(YesTick::new(Status::Failed));
		let err = Box::new(NoTick::new());

		// Put them all in a vector
		let mut children: Vec<Box<Node<AtomicBool>>> = Vec::new();
		children.push(succeed);
		children.push(fail);
		children.push(err);

		// Add them to a sequence node
		let mut seq = Sequence::new(children);

		// Tick the sequence
		let status = seq.tick(&world);

		// Drop the sequence so the nodes can do their own checks
		drop(seq);

		// Make sure we got the expected value
		assert_eq!(status, Status::Failed);
	}
}