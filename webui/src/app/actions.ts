'use server';

import { generateServiceDefinition } from '@/ai/flows/generate-service-definition-from-prompt';

export async function generateServiceDefinitionAction(prompt: string) {
  if (!prompt) {
    throw new Error('Prompt cannot be empty.');
  }

  try {
    const result = await generateServiceDefinition({ prompt });
    return result.serviceDefinition;
  } catch (error) {
    console.error('Error generating service definition:', error);
    // It's better to return a generic error message to the client
    throw new Error('Failed to generate service definition. Please try again later.');
  }
}
