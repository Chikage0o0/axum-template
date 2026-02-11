// See https://svelte.dev/docs/kit/types#app
declare global {
  namespace App {
    // interface Error {}
    // interface Locals {}
    interface PageData {
      pageTitle?: string;
      breadcrumb?: {
        section: string;
        page: string;
      };
    }
    // interface Platform {}
  }
}

export {};
