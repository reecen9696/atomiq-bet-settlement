/**
 * Utility functions for error handling
 */

/**
 * Get error message from unknown error type
 */
export function getErrorMessage(
  error: unknown,
  fallback: string = "Unknown error",
): string {
  if (error instanceof Error) {
    return error.message;
  }
  if (typeof error === "string") {
    return error;
  }
  if (error && typeof error === "object" && "message" in error) {
    return String((error as { message: unknown }).message);
  }
  return fallback;
}
