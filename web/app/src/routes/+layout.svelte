<script>
	import * as Tabs from '$lib/components/ui/tabs/index';
	import '../app.pcss';
</script>

<div class="hidden flex-col md:flex">
	<div class="border-b bg-slate-100 bg-gradient-to-t">
		<div class="flex h-16 items-center px-4">
			<div class="ml-auto flex items-center space-x-4"></div>
		</div>
	</div>

	<div class="flex-1 space-y-4 p-8 pt-6">
		<div class="flex items-center justify-between space-y-2">
			<h2 class="text-3xl font-bold tracking-tight">Febrick</h2>
			<div class="flex items-center space-x-2"></div>
		</div>
		<Tabs.Root value="classes" class="space-y-4">
			<Tabs.List>
				<Tabs.Trigger value="classes">Classes</Tabs.Trigger>
				<Tabs.Trigger value="data" disabled>Data</Tabs.Trigger>
			</Tabs.List>
			<slot />
		</Tabs.Root>
	</div>
</div>
