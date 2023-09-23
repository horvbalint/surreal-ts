export type InCategory = {
	id?: string
	name: string
	temp: {
		alma: string
	}
	tempis: Array<{
		korte: {
			mag?: Array<Date | string>
		}
	}>
	temps: Array<Date | string>
}

export type OutCategory = {
	id: string
	name: string
	temp: {
		alma: string
	}
	tempis: Array<{
		korte: {
			mag?: Array<string>
		}
	}>
	temps: Array<string>
}

export type InOffer = {
	date: Date | string
	id?: string
	price_per_package: number
	price_per_unit: number
	product: Required<InProduct>['id']
	shop: Required<InShop>['id']
}

export type OutOffer = {
	date: string
	id: string
	price_per_package: number
	price_per_unit: number
	product: OutProduct['id'] | OutProduct
	shop: OutShop['id'] | OutShop
}

export type InPackage = {
	id?: string
	name: string
}

export type OutPackage = {
	id: string
	name: string
}

export type InProduct = {
	category?: Required<InCategory>['id']
	id?: string
	name: string
	package: Required<InPackage>['id']
	unit: Required<InUnit>['id']
	unit_in_package: number
}

export type OutProduct = {
	category?: OutCategory['id'] | OutCategory
	id: string
	name: string
	package: OutPackage['id'] | OutPackage
	unit: OutUnit['id'] | OutUnit
	unit_in_package: number
}

export type InProject = {
	id?: string
	name: string
}

export type OutProject = {
	id: string
	name: string
}

export type InShop = {
	id?: string
	name: string
}

export type OutShop = {
	id: string
	name: string
}

export type InTemp = {
	category: Required<InCategory>['id']
	id?: string
	name: string
}

export type OutTemp = {
	category: OutCategory['id'] | OutCategory
	id: string
	name: string
}

export type InUnit = {
	id?: string
	name: string
}

export type OutUnit = {
	id: string
	name: string
}

export type InUser = {
	email: string
	id?: string
	password: string
}

export type OutUser = {
	email: string
	id: string
	password: string
}

export type InWork = {
	activity: string
	date: Date | string
	duration: number
	id?: string
	in: Required<InWorker>['id']
	out: Required<InProject>['id']
}

export type OutWork = {
	activity: string
	date: string
	duration: number
	id: string
	in: OutWorker['id'] | OutWorker
	out: OutProject['id'] | OutProject
}

export type InWorker = {
	id?: string
	name: string
}

export type OutWorker = {
	id: string
	name: string
}

