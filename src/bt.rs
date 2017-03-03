use std::sync::Arc;
use std::time::{Instant, Duration};
use std::thread;
use node::Node;
use status::Status;

/// Main behavior tree struct
///
/// Unless one avoids Action nodes, the world state (of type T) will be used in
/// multiple threads. Hence, all of the required markers on T.
pub struct BehaviorTree<T: Send + Sync + 'static>
{
	/// Represents the state of the world
	///
	/// A mutable reference to this object will be passed to all nodes when
	/// they are ticked.
	world: Arc<T>,

	/// Root node of the behavior tree
	root: Box<Node<T>>
}
impl<T: Send + Sync + 'static> BehaviorTree<T>
{
	/// Create a new behavior tree with the given world state object and
	/// root node.
	pub fn new(state: T, root: Box<Node<T>>) -> BehaviorTree<T>
	{
		BehaviorTree { world: Arc::new(state), root: root }
	}

	/// Create a new behavior tree with the given world state object and
	/// root node.
	///
	/// This method allows the caller to retain a copy of the world state
	pub fn with_shared_state(state: Arc<T>, root: Box<Node<T>>) -> BehaviorTree<T>
	{
		BehaviorTree { world: state, root: root }
	}

	/// Tick the behavior tree a single time
	pub fn tick(&mut self) -> Status
	{
		(*self.root).tick(&self.world)
	}

	/// Reset the tree so that it can be run again
	pub fn reset(&mut self)
	{
		(*self.root).reset()
	}

	/// Run the behavior tree until it either succeeds or fails
	///
	/// This makes no guarantees that it will run at the specified frequency. If a single
	/// tick takes longer than the alloted tick time, then it will do so silently.
	///
	/// NOTE: The only time this will return `Status::Running` is if the frequency is zero
	/// and the behavior tree is running after the first tick.
	pub fn run(&mut self, freq: f32) -> Status
	{
		// Deal with the "special" case of a zero frequency
		if freq == 0.0f32 {
			return self.tick();
		}

		// Figure out the time-per-cycle
		let cycle_dir_float = 1.0f32 / freq;
		let cycle_dir = Duration::new(cycle_dir_float as u64, (cycle_dir_float * 1000000000.0f32) as u32);

		// Now, run at the given frequency
		let mut status = Status::Running;
		while status == Status::Running {
			let now = Instant::now();
			status = self.tick();
			let elapsed = now.elapsed();

			// Sleep for the remaining amount of time
			if freq != ::std::f32::INFINITY && elapsed < cycle_dir {
				// Really, the Duration would take care of this case. However, specifying a
				// frequency of infinity means running as fast a possible. In that case, I do
				// not want to give this thread an opportunity to sleep at all
				thread::sleep(cycle_dir - elapsed);
			}
		}

		return status;
	}

	#[cfg(feature = "messages")]
	/// Creates a `Vec` with a list of `NodeMsg`s that represent this tree
	pub fn to_message(&self) -> Vec<node_message::NodeMsg>
	{
		let mut messages = Vec::new();
		(*self.root).to_message(&mut messages);

		messages
	}
}
