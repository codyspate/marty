import { someName, formatMessage, packageInfo } from '@repo/package-a';

/**
 * Generate a hello message using shared utilities
 */
export const hello = (): string => `hello ${someName()}`;

/**
 * Generate a formatted greeting
 */
export const greet = (name: string): string => formatMessage(name);

/**
 * Display information about dependencies
 */
export const showInfo = (): void => {
    console.log('Package B Info:');
    console.log('- Depends on:', packageInfo.name);
    console.log('- Dependency version:', packageInfo.version);
    console.log('- Message:', hello());
    console.log('- Greeting example:', greet('World'));
};

// Example usage when run directly
if (import.meta.url === `file://${process.argv[1]}`) {
    showInfo();
}