import _ from 'lodash';
import axios from 'axios';
// NOTE: express and moment are NOT imported - should be unreachable

async function main() {
  // Using lodash - REACHABLE (CVE-2021-23337 in 4.17.20)
  const data = { a: 1, b: 2 };
  const merged = _.merge({}, data, { c: 3 });
  console.log('Merged:', merged);

  // Using axios - REACHABLE (CVE-2021-3749 in 0.21.1)
  try {
    const response = await axios.get('https://api.github.com');
    console.log('Status:', response.status);
  } catch (e) {
    console.log('Request failed');
  }

  // NOTE: express and moment are never used
  // Their vulnerabilities should be marked as UNREACHABLE
}

main();
