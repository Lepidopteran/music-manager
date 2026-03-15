/**
 * Utility functions for workers
 */

/**
 * Send a typed message to the main thread
 */
export function sendMessage<T>(message: T) {
	postMessage(message);
}
