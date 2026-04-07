// Heavy computation worker — tests Rolldown's Web Worker bundling
self.onmessage = (e: MessageEvent<{ n: number }>) => {
    const { n } = e.data;

    // Simulate heavy structural calc: sum of story drift contributions
    let result = 0;
    for (let i = 0; i < n; i++) {
        result += Math.sin(i) * Math.cos(i) / (i + 1);
    }

    const primes: number[] = [];
    for (let i = 2; primes.length < 500; i++) {
        if ([...Array(i - 2)].map((_, k) => k + 2).every(d => i % d !== 0)) {
            primes.push(i);
        }
    }

    self.postMessage({
        result: result.toFixed(6),
        primeCount: primes.length,
        largestPrime: primes[primes.length - 1],
    });
};
