use std::f64::consts::FRAC_1_SQRT_2;

pub const fn phases<const N: usize>() -> [[f64; N]; N*N]
{
    let mut p = [[0.0; N]; N*N];
    let mut n = 0;
    while n < N*N
    {
        let mut m = n;
        let mut i = 0;
        while i < N
        {
            p[n][i] = if m % 2 == 0 {1.0} else {-1.0};
            m >>= 1;
            i += 1;
        }
        n += 1;
    }
    p
}

const fn hadamard_kernel() -> [[f64; 2]; 2]
{
    [
        [1.0, 1.0],
        [-1.0, 1.0],
    ]
}

pub fn hadamard_matrix<const N: usize>() -> [[f64; N]; N]
where
    [(); (N/2).is_power_of_two() as usize - 1]:
{
    let a1 = hadamard_kernel();
    let a0 = (1.0/N as f64).sqrt();

    let mut m = [[a0; N]; N];
    
    let n_log = N.ilog2();
    let mut i = 0;
    while i < N
    {
        let mut j = 0;
        while j < N
        {
            let mut n = 0;
            while n < n_log
            {
                m[i][j] *= a1[i/(1 << n) % 2][j/(1 << n) % 2];
                n += 1;
            }
            j += 1;
        }
        i += 1;
    }

    m
}

pub fn hadamard_feedback_matrix<const N: usize>() -> [[f64; N]; N]
where
    [(); (N/2).is_power_of_two() as usize - 1]:
{
    const fn p<const N: usize>() -> [[f64; N]; N]
    {
        let mut p = [[0.0; N]; N];
        let mut i = 0;
        while i < N/2
        {
            let mut b = 0;
            while b < 2
            {
                p[i*2 + b][i] = if (b == 0) ^ (i*2 + b >= N/2) {FRAC_1_SQRT_2} else {-FRAC_1_SQRT_2};
                p[i*2 + b][N - 1 - i] = -FRAC_1_SQRT_2;
                b += 1;
            }
            i += 1;
        }
        p
    }

    let h = hadamard_matrix();
    let p = p();
    mul_matrix(&p, &h)
}

/*pub fn householder_reflection<const N: usize>() -> [[f64; N]; N]
{
    let a0 = -2.0/N as f64;
    core::array::from_fn(|i| core::array::from_fn(|j| {
        let mut a = a0;
        if i == j
        {
            a += 1.0;
        }
        a
    }))
}*/

/*pub fn householder_feedback_matrix<const N: usize>() -> [[f64; N]; N]
where
    [(); (N/4).is_power_of_two() as usize - 1]:
{
    let a0 = (1.0/N as f64).sqrt();
    let a1 = [
        [1.0, -1.0, -1.0, -1.0],
        [-1.0, 1.0, -1.0, -1.0],
        [-1.0, -1.0, 1.0, -1.0],
        [-1.0, -1.0, -1.0, 1.0]
    ];
    core::array::from_fn(|i| core::array::from_fn(|j| {
        let mut a = a0;
        for n in 0..N.ilog2()/2
        {
            a *= a1[i/(1 << (n*2)) % 4][j/(1 << (n*2)) % 4];
        }
        a
    }))
}*/

#[test]
fn test()
{
    /*let h = hadamard_matrix::<4>();
    println!("h = {:?}", h);
    let p = [
        [1.0, 0.0, 0.0, -1.0],
        [0.0, 0.0, 0.0, 0.0],
        [0.0, 0.0, 0.0, 0.0],
        [0.0, 0.0, 0.0, 0.0]
    ];
    println!("{:?}", p.mul_matrix(&h))*/
}

pub const fn is_prime(n: usize) -> bool
{
    let n_sqrt = 1 << ((n.ilog2() + 1) / 2);
    let mut m = 2;

    while m < n_sqrt
    {
        if n % m == 0
        {
            return false
        }
        m += 1
    }

    true
}

pub const fn closest_prime(x: f64) -> usize
{
    let mut n = 2;
    let mut m = 1;
    loop
    {
        if is_prime(n)
        {
            if n as f64 > x
            {
                if (n as f64 - x) < (x - m as f64)
                {
                    return n
                }
                else
                {
                    return m
                }
            }
            m = n;
        }
        n += 1;
    }
}

pub fn primes_dist<const N: usize>(curve: f64, max: f64) -> [usize; N]
where
    [(); N - 2]:
{
    let mut scale = 0.0;
    let x = core::array::from_fn(|i| {
        let x = ((i + 1) as f64/N as f64).powf(curve);
        scale += x*x;
        x
    });
    scale = max/scale.sqrt();
    x.map(|x| {
        closest_prime(x*scale)
    })
}

#[cfg(test)]
pub const fn primes<const N: usize>(start: usize, skip: usize) -> [usize; N]
{
    let mut p = [0; N];
    let mut i = 0;
    let mut n = start;
    
    loop
    {
        let j = i/(skip + 1);
        if j >= N
        {
            break
        }
        if is_prime(n)
        {
            // n is a prime number
            if i % (skip + 1) == 0
            {
                p[j] = n;
            }
            i += 1;
        }

        n += 1;
    }
    p
}

pub const fn mul_matrix<const N: usize, const M: usize, const P: usize>(a: &[[f64; N]; M], b: &[[f64; P]; N]) -> [[f64; P]; M]
{
    let mut prod = [[0.0; P]; M];
    let mut m = 0;
    while m != M
    {
        let mut p = 0;
        while p != P
        {
            let mut n = 0;
            while n != N
            {
                prod[m][p] += a[m][n]*b[n][p];
                n += 1;
            }
            p += 1;
        }
        m += 1;
    }

    prod
}

pub fn rmul_matrix_assign_row<const M: usize>(a: &[[f64; M]; M], v: &mut [f64; M])
{
    // This is actually wrong
    *v = a.map(|rhs| {
        v.iter()
            .zip(rhs.into_iter())
            .map(|(&lhs, &rhs)| lhs*rhs)
            .sum()
    })
}