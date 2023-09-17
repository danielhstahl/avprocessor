export const intFormatter = (suffix: string) => baseFormatter(suffix, Number.parseInt)

export const floatFormatter = (suffix: string) => baseFormatter(suffix, Number.parseFloat)

export const baseFormatter = (suffix: string, parser: (v: string) => number) => ({
    formatter: (value: number | undefined) => `${value} ${suffix}`,
    parser: (value: string | undefined) => parser(value !== undefined ? value.replace(` ${suffix}`, '') : '0')
})