import { baseFormatter, intFormatter, floatFormatter } from "./inputParsers";

describe("baseFormatter", () => {
    it("returns with suffix and parsers back", () => {
        const fn = jest.fn()
        const bf = baseFormatter("hello", fn)
        expect(bf.formatter(5)).toEqual("5 hello")
        bf.parser("5 hello")
        expect(fn).toBeCalledWith("5")
    })
})

describe("intFormatter", () => {
    it("returns int with suffix and parsers back", () => {
        const bf = intFormatter("hello")
        expect(bf.formatter(5)).toEqual("5 hello")
        expect(bf.parser("5 hello")).toEqual(5)

    })
})

describe("floatFormatter", () => {
    it("returns float with suffix and parsers back", () => {
        const bf = intFormatter("hello")
        expect(bf.formatter(5.0)).toEqual("5 hello")
        expect(bf.parser("5 hello")).toEqual(5.0)

    })
})