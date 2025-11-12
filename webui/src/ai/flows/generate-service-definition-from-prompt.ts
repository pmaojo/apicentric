'use server';
/**
 * @fileOverview Generates a service definition from a natural language prompt.
 *
 * - generateServiceDefinition - A function that generates a service definition from a natural language prompt.
 * - GenerateServiceDefinitionInput - The input type for the generateServiceDefinition function.
 * - GenerateServiceDefinitionOutput - The return type for the generateServicedefinition function.
 */

import {ai} from '@/ai/genkit';
import {z} from 'genkit';

const GenerateServiceDefinitionInputSchema = z.object({
  prompt: z.string().describe('A natural language prompt describing the API service.'),
});
export type GenerateServiceDefinitionInput = z.infer<typeof GenerateServiceDefinitionInputSchema>;

const GenerateServiceDefinitionOutputSchema = z.object({
  serviceDefinition: z.string().describe('The generated service definition in YAML format.'),
});
export type GenerateServiceDefinitionOutput = z.infer<typeof GenerateServiceDefinitionOutputSchema>;

export async function generateServiceDefinition(input: GenerateServiceDefinitionInput): Promise<GenerateServiceDefinitionOutput> {
  return generateServiceDefinitionFlow(input);
}

const prompt = ai.definePrompt({
  name: 'generateServiceDefinitionPrompt',
  input: {schema: GenerateServiceDefinitionInputSchema},
  output: {schema: GenerateServiceDefinitionOutputSchema},
  prompt: `You are an expert API designer. Your task is to generate a complete and valid Apicentric service definition in YAML format based on the user's prompt.

Follow this structure precisely. The response body should be a JSON string.

Example of a good service definition:

name: user-api
version: "1.0"
description: An API for managing users.
server:
  port: 8080
  base_path: /api/v1
fixtures:
  users:
    - id: 1
      name: "Alice"
    - id: 2
      name: "Bob"
endpoints:
  - method: GET
    path: /users
    description: "Get all users"
    responses:
      200:
        content_type: application/json
        body: |
          {{{json fixtures.users}}}
  - method: POST
    path: /users
    description: "Create a new user"
    responses:
      201:
        content_type: application/json
        body: |
          {
            "id": {{faker "datatype.number" min=100 max=999}},
            "name": "{{request.body.name}}",
            "status": "created"
          }
      400:
        condition: "{{not request.body.name}}"
        content_type: application/json
        body: |
          {
            "error": "Name is required"
          }

Now, generate a service definition for the following prompt:
"{{{prompt}}}"`,
});

const generateServiceDefinitionFlow = ai.defineFlow(
  {
    name: 'generateServiceDefinitionFlow',
    inputSchema: GenerateServiceDefinitionInputSchema,
    outputSchema: GenerateServiceDefinitionOutputSchema,
  },
  async input => {
    const {output} = await prompt(input);
    return output!;
  }
);
