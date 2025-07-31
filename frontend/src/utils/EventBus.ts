/**
 * Type definition for a callback function.
 * 
 * @template T - The type of the payload passed when emitting the event.
 */
type Callback<T = any> = (payload: T) => void;

/**
 * A simple in-memory publish/subscribe event bus.
 * 
 * Allows registering listeners for named events (`on`), removing them (`off`), 
 * and broadcasting (`emit`) with an optional payload.
 * 
 * Usage:
 * ```ts
 * import EventBus from './EventBus';
 * 
 * const listener = (payload) => { console.log(payload); };
 * EventBus.on('my-event', listener);
 * 
 * EventBus.emit('my-event', { foo: 'bar' });
 * 
 * EventBus.off('my-event', listener);
 * ```
 */
class EventBus {
  /**
   * Internal map of event names to a set of listeners.
   */
  private listeners = new Map<string, Set<Callback>>();

  /**
   * Register a listener for a specific event.
   *
   * @param event - The name of the event to listen for.
   * @param callback - The function to call when the event is emitted.
   */
  on<T = any>(event: string, callback: Callback<T>) {
    if (!this.listeners.has(event)) {
      this.listeners.set(event, new Set());
    }
    this.listeners.get(event)!.add(callback);
  }

  /**
   * Remove a previously registered listener for a specific event.
   *
   * @param event - The name of the event to stop listening to.
   * @param callback - The callback to remove.
   */
  off<T = any>(event: string, callback: Callback<T>) {
    this.listeners.get(event)?.delete(callback);
  }

  /**
   * Emit an event, optionally passing a payload to all registered listeners.
   *
   * @param event - The name of the event to emit.
   * @param payload - Optional data to pass to the listeners.
   */
  emit<T = any>(event: string, payload?: T) {
    this.listeners.get(event)?.forEach((callback) => {
      callback(payload);
    });
  }
}

export default new EventBus();
