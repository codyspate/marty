/**
 * Returns a greeting name
 */
export const someName = (): string => "some name";

/**
 * Utility function to format messages
 */
export const formatMessage = (name: string): string => `Hello, ${name}!`;

/**
 * Package version and info
 */
export const packageInfo = {
    name: '@repo/package-a',
    version: '1.0.0',
    description: 'Shared utilities package'
} as const;