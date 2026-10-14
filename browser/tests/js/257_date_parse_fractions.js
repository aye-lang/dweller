// Fractional seconds of any length: V8 and SpiderMonkey drop the digits
// past the milliseconds and scale short fractions (".5" is 500 ms).
// DuckDuckGo prints dates like "2026-07-20T00:00:00.0000000".
console.log(Date.parse('2026-07-20T00:00:00.0000000Z'));
console.log(new Date('2026-09-23T12:34:56.1234567Z').toISOString());
console.log(Date.parse('2020-01-02T03:04:05.5Z'), Date.parse('2020-01-02T03:04:05.25Z'));
console.log(new Date('2020-01-02T03:04:05.9999999+02:00').toISOString());
console.log(isNaN(Date.parse('2020-01-02T03:04:05.Z')), isNaN(Date.parse('2020-01-02T03:04:05.12x')));
