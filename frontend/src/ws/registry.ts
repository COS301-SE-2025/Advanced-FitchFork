// src/ws/registry.ts
import type { EventPayloadMap } from './types';

export type EventName = keyof EventPayloadMap;

/** Normalized event callback */
export type EventHandler<Evt extends EventName = EventName> = (
  payload: EventPayloadMap[Evt],
  meta: { topic: string; ts: string; v?: number | null }
) => void;

/**
 * Holds per-topic listeners and a ref-count so we only SUBSCRIBE once
 * per topic and UNSUBSCRIBE when the last consumer leaves.
 */
export class SubscriptionRegistry {
  private topics = new Map<
    string,
    {
      refCount: number;
      listeners: Map<EventName, Set<EventHandler>>;
      lastVersion?: number;
    }
  >();

  /** Add interest in (topicPath, events); returns true if this was the first ref for that topic. */
  add(topicPath: string, events: EventName[], handler: EventHandler): boolean {
    let entry = this.topics.get(topicPath);
    if (!entry) {
      entry = { refCount: 0, listeners: new Map<EventName, Set<EventHandler>>() };
      this.topics.set(topicPath, entry);
    }

    entry.refCount++;
    for (const ev of events) {
      const set = entry.listeners.get(ev) ?? new Set<EventHandler>();
      set.add(handler);
      entry.listeners.set(ev, set);
    }
    return entry.refCount === 1;
  }

  /**
   * Remove interest in (topicPath, events) for a specific handler.
   * Returns true if the topic's refCount dropped to zero (caller should UNSUBSCRIBE).
   */
  remove(topicPath: string, events: EventName[], handler: EventHandler): boolean {
    const entry = this.topics.get(topicPath);
    if (!entry) return false;

    for (const ev of events) {
      const set = entry.listeners.get(ev);
      if (set) {
        set.delete(handler);
        if (set.size === 0) entry.listeners.delete(ev);
      }
    }

    entry.refCount = Math.max(0, entry.refCount - 1);
    if (entry.refCount === 0) {
      this.topics.delete(topicPath);
      return true;
    }
    return false;
  }

  /** Dispatch an incoming event to all listeners of (topicPath, eventName). */
  dispatch<Evt extends EventName>(
    eventName: Evt,
    topicPath: string,
    payload: EventPayloadMap[Evt],
    ts: string,
    v?: number | null
  ): void {
    const entry = this.topics.get(topicPath);
    if (!entry) return;

    if (typeof v === 'number') entry.lastVersion = v;

    const set = entry.listeners.get(eventName);
    if (!set || set.size === 0) return;

    const meta = { topic: topicPath, ts, v };
    for (const fn of set) {
      // Type-safe by construction
      (fn as EventHandler<Evt>)(payload, meta);
    }
  }

  /** Returns the last seen version for a topic (useful for subscribe since=...). */
  lastSeenVersion(topicPath: string): number | undefined {
    return this.topics.get(topicPath)?.lastVersion;
  }

  /** True if we still have listeners for this topic. */
  has(topicPath: string): boolean {
    return this.topics.has(topicPath);
  }

  /** Current refCount (mainly for debugging). */
  refCount(topicPath: string): number {
    return this.topics.get(topicPath)?.refCount ?? 0;
  }
}
