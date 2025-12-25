// Test: Multi-field form prompt
// Usage: npx tsx tests/sdk/test-fields.ts

import '../../scripts/kit-sdk.ts';

// Test with simple string fields
const simpleValues = await fields([
  'Name',
  'Email',
  'Phone',
]);

console.log('Simple field values:', simpleValues);

// Test with detailed field definitions
const detailedValues = await fields([
  { name: 'username', label: 'Username', placeholder: 'Enter your username' },
  { name: 'password', label: 'Password', type: 'password', placeholder: 'Enter password' },
  { name: 'email', label: 'Email Address', type: 'email', value: 'user@example.com' },
  { name: 'age', label: 'Age', type: 'number' },
  { name: 'birthday', label: 'Birthday', type: 'date' },
  { name: 'website', label: 'Website', type: 'url', placeholder: 'https://example.com' },
]);

console.log('Detailed field values:', detailedValues);

// Test custom HTML form
const formData = await form(`
  <form>
    <label for="firstName">First Name:</label>
    <input type="text" id="firstName" name="firstName" />
    
    <label for="lastName">Last Name:</label>
    <input type="text" id="lastName" name="lastName" />
    
    <label for="favoriteColor">Favorite Color:</label>
    <input type="color" id="favoriteColor" name="favoriteColor" />
  </form>
`);

console.log('Form data:', formData);
