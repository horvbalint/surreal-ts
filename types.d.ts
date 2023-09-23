export interface InCategory {
	id?: string
	name: string
}

export interface OutCategory {
	id: string
	name: string
}

export interface InOffer {
	date: Date | string
	id?: string
	price_per_package: number
	price_per_unit: number
	product: InProduct
	shop: InShop
}

export interface OutOffer {
	date: string
	id: string
	price_per_package: number
	price_per_unit: number
	product: OutProduct
	shop: OutShop
}

export interface InPackage {
	id?: string
	name: string
}

export interface OutPackage {
	id: string
	name: string
}

export interface InProduct {
	category?: InCategory
	id?: string
	name: string
	package: InPackage
	unit: InUnit
	unit_in_package: number
}

export interface OutProduct {
	category?: OutCategory
	id: string
	name: string
	package: OutPackage
	unit: OutUnit
	unit_in_package: number
}

export interface InProject {
	id?: string
	name: string
}

export interface OutProject {
	id: string
	name: string
}

export interface InShop {
	id?: string
	name: string
}

export interface OutShop {
	id: string
	name: string
}

export interface InUnit {
	id?: string
	name: string
}

export interface OutUnit {
	id: string
	name: string
}

export interface InUser {
	email: string
	id?: string
	password: string
}

export interface OutUser {
	email: string
	id: string
	password: string
}

export interface InWork {
	activity: string
	date: Date | string
	duration: number
	id?: string
	in: InWorker
	out: InProject
}

export interface OutWork {
	activity: string
	date: string
	duration: number
	id: string
	in: OutWorker
	out: OutProject
}

export interface InWorker {
	id?: string
	name: string
}

export interface OutWorker {
	id: string
	name: string
}

